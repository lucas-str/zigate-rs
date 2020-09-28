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
    responses::{Response},
};

pub struct Zigate {
    data: Arc<Mutex<ZigateData>>,
    sender: UartSender,
    pathbuf: PathBuf,
    version: Option<String>,
}

struct ZigateData {
    pub version: Option<String>,
    pub last_resp: HashMap<MessageType, Command>,
}

impl Zigate {
    pub fn new(path: &Path) -> Self {
        let pathbuf = path.to_path_buf();
        let sender = UartSender::new(&path);
        let data = ZigateData {
            version: None,
            last_resp: HashMap::new(),
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

    fn wait_for_response(&self, msg_type: &MessageType) -> Option<Command> {
        for _ in 0..50 {
            {
                let data = self.data.lock().unwrap();
                if let Some(cmd) = data.last_resp.get(msg_type) {
                    return Some(cmd.clone())
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        None
    }

    pub fn get_version(&mut self) -> Result<String, ()> {
        {
            if let Some(version) = &self.version {
                return Ok(version.clone())
            }
        }
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
}

fn recv_fn(rx: Receiver<Command>, _sender: UartSender, data: Arc<Mutex<ZigateData>>) {
    loop {
        match rx.recv() {
            Ok(cmd) => {
                println!("recv: {}", cmd);
                let mut data = data.lock().unwrap();
                let msg_type = MessageType::from_u16(cmd.msg_type);
                let data_cmd = cmd.clone();
                data.last_resp.insert(msg_type, data_cmd);
                //let response = ResponseBox::from_command(&cmd);
                //match response {
                //    ResponseBox::VersionListBox(version_list) => {
                //        data.version = Some(format!("{}.{}", version_list.major,
                //                                    version_list.installer).to_owned());
                //    },
                //    _ => {},
                //}
            },
            Err(err) => println!("error: {}", err),
        }
    }
}
