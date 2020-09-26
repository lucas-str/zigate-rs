use rppal::uart::{Parity, Uart};
use std::path::{Path, PathBuf};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::marker::Send;

use crate::command;

pub struct Zigate {
    pathbuf: PathBuf,
    cb: Option<Arc<Mutex<ResponseCallback>>>,
    uart: Uart,
}

type ResponseCallback = fn(&command::Command);

fn recv_thread(pathbuf: PathBuf, cb: Option<Arc<Mutex<ResponseCallback>>>) {
    let path = pathbuf.as_path();
    let mut uart = Uart::with_path(path, 115_200, Parity::None, 8, 1).unwrap();
    uart.set_read_mode(1, Duration::default()).unwrap();
    let mut buf = [0u8; 1];
    let mut msg = Vec::new();
    loop {
        if uart.read(&mut buf).unwrap() > 0 {
            if buf[0] == 1 {
                msg.clear();
            }
            msg.push(buf[0]);
            if buf[0] == 3 {
                match command::Command::from_raw(&msg) {
                    Ok(cmd) => {
                        //println!("Received: {}", cmd);
                        //let response = responses::parse_response(&cmd);
                        //println!("{}", response.to_string());
                        //let cb = cb.lock().unwrap();
                        if let Some(ref cb) = cb {
                            let cb = cb.lock().unwrap();
                            cb(&cmd);
                        }
                    },
                    Err(err) => println!("Error: {}\n{:?}", err, msg),
                }
                msg.clear();
            }
        }
    }
}

type ResponseCallback2<T> = fn(&command::Command, &mut T);

fn recv_thread2<T>(pathbuf: PathBuf, cb: ResponseCallback2<T>, user_data: &mut Arc<Mutex<T>>) {
    let path = pathbuf.as_path();
    let mut uart = Uart::with_path(path, 115_200, Parity::None, 8, 1).unwrap();
    uart.set_read_mode(1, Duration::default()).unwrap();
    let mut buf = [0u8; 1];
    let mut msg = Vec::new();
    loop {
        if uart.read(&mut buf).unwrap() > 0 {
            if buf[0] == 1 {
                msg.clear();
            }
            msg.push(buf[0]);
            if buf[0] == 3 {
                match command::Command::from_raw(&msg) {
                    Ok(cmd) => {
                        let mut user_data = user_data.lock().unwrap();
                        cb(&cmd, &mut user_data);
                    },
                    Err(err) => println!("Error: {}\n{:?}", err, msg),
                }
                msg.clear();
            }
        }
    }
}

impl Zigate {
    pub fn new(path: &Path) -> Zigate {
        let pathbuf = path.clone().to_path_buf();
        let path = pathbuf.as_path();
        let uart = Uart::with_path(path, 115_200, Parity::None, 8, 1).unwrap();
        Zigate {
            pathbuf,
            cb: None,
            uart,
        }
    }

    pub fn set_response_callback(&mut self, cb: ResponseCallback) {
        let cb = Arc::new(Mutex::new(cb));
        self.cb = Some(cb);
    }

    pub fn start(&self) -> thread::JoinHandle<()> {
        let pathbuf = PathBuf::from(&self.pathbuf);
        match &self.cb {
            Some(cb) => {
                let cb = cb.clone();
                thread::spawn(move || recv_thread(pathbuf, Some(cb)))
            },
            None => thread::spawn(move || recv_thread(pathbuf, None))
        }
        //match &self.cb {
        //    Some(cb) => {
        //        let pathbuf = PathBuf::from(&self.pathbuf);
        //        let cb = Arc::clone(&cb);
        //        Ok(thread::spawn(move || recv_thread(pathbuf, cb)))
        //    },
        //    None => return Err(())
        //}
    }

    pub fn start2<T: 'static + Send>(&self, cb: ResponseCallback2<T>,
                     user_data: &Arc<Mutex<T>>) -> thread::JoinHandle<()> {
        let pathbuf = PathBuf::from(&self.pathbuf);
        let mut user_data = user_data.clone();
        thread::spawn(move || recv_thread2(pathbuf, cb, &mut user_data))
    }

    pub fn send(&mut self, cmd: &command::Command) {
        self.uart.set_write_mode(true).unwrap();
        let wl = self.uart.write(&cmd.serialize()).unwrap();
        debug!("Sent {} bytes", wl);
    }
}
