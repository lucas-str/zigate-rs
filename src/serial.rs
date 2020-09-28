use rppal::uart::{Parity, Uart};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::path::Path;
use std::time::Duration;

use crate::command;

pub struct UartSender {
    uart: Uart,
}

impl UartSender {
    pub fn new(path: &Path) -> Self {
        let mut uart = Uart::with_path(path, 115_200, Parity::None, 8, 1).unwrap();
        uart.set_write_mode(true).unwrap();
        Self {
            uart,
        }
    }

    pub fn send(&mut self, cmd: &command::Command) {
        let wl = self.uart.write(&cmd.serialize()).unwrap();
        debug!("Sent {} bytes", wl);
    }
}

fn recv_commands(mut uart: Uart, tx: Sender<command::Command>) {
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
                        tx.send(cmd).unwrap();
                    },
                    Err(err) => println!("Error: {}\n{:?}", err, msg),
                }
                msg.clear();
            }
        }
    }
}

pub fn uart_recver(path: &Path) -> Receiver<command::Command> {
    let (tx, rx) = channel::<command::Command>();
    let mut uart = Uart::with_path(path, 115_200, Parity::None, 8, 1).unwrap();
    uart.set_read_mode(1, Duration::default()).unwrap();
    thread::spawn(move || {
        recv_commands(uart, tx);
    });
    rx
}
