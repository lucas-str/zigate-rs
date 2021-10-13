use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    cluster::Cluster,
    command::{Command, MessageType},
    commands,
    device::Device,
    responses,
    responses::{Response, ResponseBox},
    serial::{uart_recver, UartSender},
};

pub struct Zigate {
    data: Arc<Mutex<ZigateData>>,
    sender: UartSender,
    pathbuf: PathBuf,
    version: Option<String>,
}

struct ZigateData {
    pub last_resp: HashMap<MessageType, Command>,
    pub last_status: HashMap<MessageType, Command>,
    pub exp_resp: u16,
    pub devices: HashMap<u16, Device>,
}

impl Zigate {
    pub fn new(path: &Path) -> Self {
        let pathbuf = path.to_path_buf();
        let sender = UartSender::new(&path);
        let data = ZigateData {
            last_resp: HashMap::new(),
            last_status: HashMap::new(),
            exp_resp: 0,
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

    fn wait_for_responses(&mut self) {
        for _ in 0..50 {
            {
                let data = self.data.lock().unwrap();
                if data.exp_resp == 0 {
                    return;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        let mut data = self.data.lock().unwrap();
        error!("{} message(s) lost", data.exp_resp);
        data.exp_resp = 0;
    }

    fn wait_for_response(&mut self, msg_type: &MessageType) -> Option<Command> {
        for _ in 0..50 {
            {
                let mut data = self.data.lock().unwrap();
                if let Some(cmd) = data.last_resp.remove(msg_type) {
                    return Some(cmd);
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        None
    }

    fn send_and_wait(&mut self, cmd: &Command, msg_type: &MessageType) {
        self.remove_last_response(msg_type);
        self.send(cmd);
        self.wait_for_response(msg_type);
    }

    fn remove_last_response(&mut self, msg_type: &MessageType) -> Option<Command> {
        let mut data = self.data.lock().unwrap();
        let last_status = data.last_status.remove(msg_type);
        match data.last_resp.remove(msg_type) {
            Some(last_resp) => return Some(last_resp),
            None => return last_status,
        };
    }

    pub fn permit_join(&mut self, interval: u8) -> Result<(), ()> {
        self.send(&commands::permit_join_request(0xfffc, interval, 0));
        Ok(())
    }

    pub fn get_version(&mut self) -> Result<String, ()> {
        {
            if let Some(version) = &self.version {
                return Ok(version.clone());
            }
        }
        self.remove_last_response(&MessageType::VersionList);
        self.send(&commands::get_version());
        match self.wait_for_response(&MessageType::VersionList) {
            Some(cmd) => {
                if let Ok(version_list) = responses::VersionList::from_command(&cmd) {
                    let version =
                        String::from(format!("{}.{}", version_list.major, version_list.installer));
                    self.version = Some(version.clone());
                    return Ok(version);
                }
                Err(())
            }
            None => Err(()),
        }
    }

    pub fn get_devices(&mut self) -> HashMap<u16, Device> {
        self.send_and_wait(&commands::get_devices_list(), &MessageType::DevicesList);
        self.wait_for_responses();
        let data = self.data.lock().unwrap();
        data.devices.clone()
    }

    pub fn get_onoff(&mut self, address: u16, endpoint: u8) -> Result<bool, ()> {
        let cmd = commands::simple_read_attribut_request(address, endpoint, 6, 0);
        self.send_and_wait(&cmd, &MessageType::ReportIndividualAttributResponse);
        let data = self.data.lock().unwrap();
        if let Some(device) = data.devices.get(&address) {
            if let Some(endpoint) = device.get_endpoint(endpoint) {
                for cluster in endpoint.get_in_clusters() {
                    if let Cluster::GeneralOnOff(cluster) = cluster {
                        return Ok(cluster.onoff);
                    }
                }
            }
        }
        Err(())
    }

    pub fn onoff(&mut self, address: u16, endpoint: u8, onoff: bool) {
        let cmd = commands::action_onoff(address, 1, endpoint, onoff as u8);
        self.send(&cmd);
    }

    pub fn get_level(&mut self, address: u16, endpoint: u8) -> Result<u8, ()> {
        let cmd = commands::simple_read_attribut_request(address, endpoint, 8, 0);
        self.send_and_wait(&cmd, &MessageType::ReportIndividualAttributResponse);
        let data = self.data.lock().unwrap();
        if let Some(device) = data.devices.get(&address) {
            if let Some(endpoint) = device.get_endpoint(endpoint) {
                for cluster in endpoint.get_in_clusters() {
                    if let Cluster::GeneralLevelControl(cluster) = cluster {
                        return Ok(cluster.current_level);
                    }
                }
            }
        }
        Err(())
    }

    pub fn move_to_level(
        &mut self,
        address: u16,
        endpoint: u8,
        on: bool,
        level: u8,
        transition_time: u16,
    ) {
        let on = on as u8;
        let cmd = commands::action_move_onoff(address, 1, endpoint, on, level, transition_time);
        self.send(&cmd);
    }

    pub fn get_color_capabilities(
        &mut self,
        address: u16,
        endpoint: u8,
    ) -> Result<ColorCapabilities, ()> {
        let cmd = commands::simple_read_attribut_request(address, endpoint, 0x0300, 0);
        self.send_and_wait(&cmd, &MessageType::ReportIndividualAttributResponse);
        Err(())
    }

    pub fn get_color_hue(&mut self, address: u16, endpoint: u8) -> Result<u8, ()> {
        let cmd = commands::simple_read_attribut_request(address, endpoint, 0x0300, 0);
        self.send_and_wait(&cmd, &MessageType::ReportIndividualAttributResponse);
        let data = self.data.lock().unwrap();
        if let Some(device) = data.devices.get(&address) {
            if let Some(endpoint) = device.get_endpoint(endpoint) {
                for cluster in endpoint.get_in_clusters() {
                    if let Cluster::LightingColorControl(cluster) = cluster {
                        if let Some(hue) = cluster.current_hue {
                            return Ok(hue);
                        }
                    }
                }
            }
        }
        return Err(());
    }

    pub fn get_color_saturation(&mut self, address: u16, endpoint: u8) -> Result<u8, ()> {
        let cmd = commands::simple_read_attribut_request(address, endpoint, 0x0300, 1);
        self.send_and_wait(&cmd, &MessageType::ReportIndividualAttributResponse);
        let data = self.data.lock().unwrap();
        if let Some(device) = data.devices.get(&address) {
            if let Some(endpoint) = device.get_endpoint(endpoint) {
                for cluster in endpoint.get_in_clusters() {
                    if let Cluster::LightingColorControl(cluster) = cluster {
                        if let Some(sat) = cluster.current_saturation {
                            return Ok(sat);
                        }
                    }
                }
            }
        }
        return Err(());
    }

    pub fn get_color(&mut self, address: u16, endpoint: u8) -> Result<(u16, u16), ()> {
        let cmd = commands::simple_read_attribut_request(address, endpoint, 0x0300, 3);
        self.send_and_wait(&cmd, &MessageType::ReportIndividualAttributResponse);
        let cmd = commands::simple_read_attribut_request(address, endpoint, 0x0300, 4);
        self.send_and_wait(&cmd, &MessageType::ReportIndividualAttributResponse);
        let data = self.data.lock().unwrap();
        if let Some(device) = data.devices.get(&address) {
            if let Some(endpoint) = device.get_endpoint(endpoint) {
                for cluster in endpoint.get_in_clusters() {
                    if let Cluster::LightingColorControl(cluster) = cluster {
                        if let (Some(x), Some(y)) = (cluster.current_x, cluster.current_y) {
                            return Ok((x, y));
                        }
                    }
                }
            }
        }
        return Err(());
    }

    pub fn move_to_hue(
        &mut self,
        address: u16,
        endpoint: u8,
        hue: u8,
        direction: u8,
        transition_time: u16,
    ) {
        let cmd =
            commands::action_move_to_hue(address, 1, endpoint, hue, direction, transition_time);
        self.send(&cmd);
    }

    pub fn move_to_saturation(
        &mut self,
        address: u16,
        endpoint: u8,
        saturation: u8,
        transition_time: u16,
    ) {
        let cmd =
            commands::action_move_to_saturation(address, 1, endpoint, saturation, transition_time);
        self.send(&cmd);
    }

    pub fn move_to_hue_and_saturation(
        &mut self,
        address: u16,
        endpoint: u8,
        hue: u8,
        saturation: u8,
        transition_time: u16,
    ) {
        let cmd = commands::action_move_to_hue_and_saturation(
            address,
            1,
            endpoint,
            hue,
            saturation,
            transition_time,
        );
        self.send(&cmd);
    }

    pub fn move_to_color(
        &mut self,
        address: u16,
        endpoint: u8,
        x: u16,
        y: u16,
        transition_time: u16,
    ) {
        let cmd = commands::action_move_to_color(address, 1, endpoint, x, y, transition_time);
        self.send(&cmd);
    }

    pub fn get_color_temp(&mut self, address: u16, endpoint: u8) -> Result<u16, ()> {
        let cmd = commands::simple_read_attribut_request(address, endpoint, 0x0300, 7);
        self.send_and_wait(&cmd, &MessageType::ReportIndividualAttributResponse);
        let data = self.data.lock().unwrap();
        if let Some(device) = data.devices.get(&address) {
            if let Some(endpoint) = device.get_endpoint(endpoint) {
                for cluster in endpoint.get_in_clusters() {
                    if let Cluster::LightingColorControl(cluster) = cluster {
                        if let Some(temp) = cluster.color_temperature {
                            return Ok(temp);
                        }
                    }
                }
            }
        }
        Err(())
    }

    pub fn move_to_color_temp(
        &mut self,
        address: u16,
        endpoint: u8,
        color_temp: u16,
        transition_time: u16,
    ) {
        let cmd =
            commands::action_move_color_temp(address, 1, endpoint, color_temp, transition_time);
        self.send(&cmd);
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
                info!("recv: {}", response.to_string());
                match response {
                    ResponseBox::StatusBox(_) => {
                        let msg_type = MessageType::from_u16(cmd.msg_type);
                        data.last_status.insert(msg_type, cmd.clone());
                    }
                    ResponseBox::DeviceAnnounceBox(msg) => {
                        if !data.devices.contains_key(&msg.short_address) {
                            let device = Device::from_device_announce(&msg);
                            data.devices.insert(device.short_address, device);
                            sender.send(&commands::active_endpoint_request(msg.short_address));
                            data.exp_resp += 1;
                        }
                    }
                    ResponseBox::DevicesListBox(msg) => {
                        for device in msg.devices {
                            if !data.devices.contains_key(&device.short_address) {
                                let device = Device::from_devices_list_elem(device);
                                sender
                                    .send(&commands::active_endpoint_request(device.short_address));
                                data.exp_resp += 1;
                                data.devices.insert(device.short_address, device);
                            }
                        }
                    }
                    ResponseBox::ActiveEndpointsBox(msg) => {
                        if let Some(device) = data.devices.get_mut(&msg.address) {
                            device.add_endpoints(&msg.endpoint_list);
                            for endpoint in msg.endpoint_list {
                                sender.send(&commands::simple_descriptor_request(
                                    msg.address,
                                    endpoint,
                                ));
                                data.exp_resp += 1;
                            }
                            data.exp_resp -= 1;
                        }
                    }
                    ResponseBox::SimpleDescriptorResponseBox(msg) => {
                        if let Some(device) = data.devices.get_mut(&msg.address) {
                            device.set_endpoints_clusters(&msg);
                            data.exp_resp -= 1;
                        }
                    }
                    ResponseBox::ReadAttributeResponseBox(msg) => {
                        if let Some(device) = data.devices.get_mut(&msg.src_addr) {
                            device.update_cluster(&msg);
                        }
                    }
                    ResponseBox::ReportIndividualAttributResponseBox(msg) => {
                        if let Some(device) = data.devices.get_mut(&msg.src_addr) {
                            device.update_cluster(&msg);
                        }
                    }
                    _ => {}
                }
            }
            Err(err) => error!("error: {}", err),
        }
    }
}
