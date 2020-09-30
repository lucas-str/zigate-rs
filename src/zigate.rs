use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::Receiver,
    },
    thread,
    time::Duration,
    collections::HashMap,
};

use crate::{
    serial::{UartSender, uart_recver},
    command::{Command, MessageType},
    commands,
    responses,
    responses::{Response, ResponseBox},
    //device::Device,
};

use crate::responses::Device as RespDevice;

pub struct Zigate {
    data: Arc<Mutex<ZigateData>>,
    sender: UartSender,
    pathbuf: PathBuf,
    version: Option<String>,
}

struct ZigateData {
    pub version: Option<String>,
    pub last_resp: HashMap<MessageType, Command>,
    pub last_status: HashMap<MessageType, Command>,
    pub devices: HashMap<u16, Device>,
}

impl Zigate {
    pub fn new(path: &Path) -> Self {
        let pathbuf = path.to_path_buf();
        let sender = UartSender::new(&path);
        let data = ZigateData {
            version: None,
            last_resp: HashMap::new(),
            last_status: HashMap::new(),
            devices: HashMap::new(),
        };
        let data = Arc::new(Mutex::new(data));
        let version = None;
        Self {
            sender,
            data,
            pathbuf,
            version,
        }
    }

    pub fn start(&self) -> thread::JoinHandle<()> {
        let path = self.pathbuf.as_path();
        let rx = uart_recver(path);
        let rx_sender = UartSender::new(path);
        let data = self.data.clone();
        thread::spawn(move || recv_fn(rx, rx_sender, data))
    }

    pub fn send(&mut self, cmd: &Command) {
        self.sender.send(cmd);
    }

    fn wait_for_response(&mut self, msg_type: &MessageType) -> Option<Command> {
        for _ in 0..50 {
            {
                let mut data = self.data.lock().unwrap();
                if let Some(cmd) = data.last_resp.remove(msg_type) {
                    return Some(cmd)
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        None
    }

    fn remove_last_response(&mut self, msg_type: &MessageType) -> Option<Command> {
        let mut data = self.data.lock().unwrap();
        let last_status = data.last_status.remove(msg_type);
        match data.last_resp.remove(msg_type) {
            Some(last_resp) => return Some(last_resp),
            None => return last_status,
        };
    }

    pub fn get_version(&mut self) -> Result<String, ()> {
        {
            if let Some(version) = &self.version {
                return Ok(version.clone())
            }
        }
        self.remove_last_response(&MessageType::VersionList);
        self.send(&commands::get_version());
        match self.wait_for_response(&MessageType::VersionList) {
            Some(cmd) => {
                if let Ok(version_list) = responses::VersionList::from_command(&cmd) {
                    let version = String::from(format!("{}.{}", version_list.major, version_list.installer));
                    self.version = Some(version.clone());
                    return Ok(version)
                }
                Err(())
            },
            None => Err(()),
        }
    }

    pub fn get_devices(&mut self) -> HashMap<u16, Device> {
        self.remove_last_response(&MessageType::DevicesList);
        self.send(&commands::get_devices_list());
        self.wait_for_response(&MessageType::DevicesList);
        let data = self.data.lock().unwrap();
        data.devices.clone()
    }
}

fn recv_fn(rx: Receiver<Command>, mut sender: UartSender, data: Arc<Mutex<ZigateData>>) {
    loop {
        match rx.recv() {
            Ok(cmd) => {
                let mut data = data.lock().unwrap();
                let msg_type = MessageType::from_u16(cmd.msg_type);
                let data_cmd = cmd.clone();
                data.last_resp.insert(msg_type, data_cmd);
                let response = ResponseBox::from_command(&cmd);
                println!("recv: {}", response.to_string());
                match response {
                    ResponseBox::StatusBox(_) => {
                        let msg_type = MessageType::from_u16(cmd.msg_type);
                        data.last_status.insert(msg_type, cmd.clone());
                    },
                    ResponseBox::DeviceAnnounceBox(msg) => {
                        if !data.devices.contains_key(&msg.short_address) {
                            let device = Device::from_device_announce(&msg);
                            data.devices.insert(device.short_address, device);
                            sender.send(&commands::active_endpoint_request(msg.short_address));
                        }
                    },
                    ResponseBox::DevicesListBox(msg) => {
                        for device in msg.devices {
                            if !data.devices.contains_key(&device.short_address) {
                                let device = Device::from_devices_list_elem(device);
                                sender.send(
                                    &commands::active_endpoint_request(device.short_address));
                                data.devices.insert(device.short_address, device);
                            }
                        }
                    },
                    ResponseBox::ActiveEndpointsBox(msg) => {
                        if let Some(device) = data.devices.get_mut(&msg.address) {
                            device.add_endpoints(&msg.endpoint_list);
                            for endpoint in msg.endpoint_list {
                                sender.send(&commands::simple_descriptor_request(msg.address,
                                                                                 endpoint));
                            }
                        }
                    },
                    ResponseBox::SimpleDescriptorResponseBox(msg) => {
                        if let Some(device) = data.devices.get_mut(&msg.address) {
                            for mut endpoint in device.endpoints.as_mut_slice() {
                                endpoint.set_in_clusters(msg.in_cluster_list.clone());
                                endpoint.set_out_clusters(msg.out_cluster_list.clone());
                            }
                        }
                    },
                    _ => {},
                }
            },
            Err(err) => println!("error: {}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    pub id: Option<u8>,
    pub short_address: u16,
    pub ieee_address: u64,
    pub power_source: Option<bool>,
    pub link_quality: Option<u8>,
    pub endpoints: Vec<Endpoint>,
}

impl Device {
    pub fn new(short_address: u16, ieee_address: u64) -> Self {
        Self {
            short_address,
            ieee_address,
            id: None,
            power_source: None,
            link_quality: None,
            endpoints: Vec::new(),
        }
    }

    pub fn from_devices_list_elem(device: RespDevice) -> Self {
        Self {
            short_address: device.short_address,
            ieee_address: device.ieee_address,
            id: Some(device.id),
            power_source: Some(device.power_source),
            link_quality: Some(device.link_quality),
            endpoints: Vec::new(),
        }
    }

    pub fn from_device_announce(msg: &responses::DeviceAnnounce) -> Self {
        Self {
            id: None,
            short_address: msg.short_address,
            ieee_address: msg.ieee_address,
            power_source: None,
            link_quality: None,
            endpoints: Vec::new(),
        }
    }

    pub fn add_endpoints(&mut self, endpoints: &Vec<u8>) {
        for endpoint in endpoints {
            self.endpoints.push(Endpoint::new(*endpoint));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    id: u8,
    in_clusters: Vec<u16>,
    out_clusters: Vec<u16>,
}

impl Endpoint {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            in_clusters: Vec::new(),
            out_clusters: Vec::new(),
        }
    }

    pub fn set_in_clusters(&mut self, clusters: Vec<u16>) {
        self.in_clusters = clusters;
    }

    pub fn set_out_clusters(&mut self, clusters: Vec<u16>) {
        self.out_clusters = clusters;
    }
}
