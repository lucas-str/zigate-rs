#[macro_use] extern crate log;

use std::path::Path;
use std::thread;
use std::time::Duration;

use rppal::uart::{Parity, Uart};

mod command;

fn recv() {
    let ama0 = Path::new("/dev/ttyAMA0");
    info!("Receiving from {}", ama0.display());

    let mut uart = Uart::with_path(&ama0, 115_200, Parity::None, 8, 1).unwrap();

    uart.set_read_mode(1, Duration::default()).unwrap();

    let mut buf = [0u8; 1];
    let mut msg = Vec::new();
    loop {
        if uart.read(&mut buf).unwrap() > 0 {
            if buf[0] == 1 {
                msg.clear();
            } else if buf[0] == 3 {
                match command::Command::from_raw(&msg) {
                    Ok(cmd) => println!("{:?}", cmd),
                    Err(msg) => println!("Error: {}", msg),
                }
            } else {
                msg.push(buf[0]);
            }
        }
    }
}

fn send(cmd: & Vec<u8>) {
    let ama0 = Path::new("/dev/ttyAMA0");
    info!("Send to {}", ama0.display());

    let mut uart = Uart::with_path(&ama0, 115_200, Parity::None, 8, 1).unwrap();

    uart.set_write_mode(true).unwrap();

    let wl = uart.write(cmd).unwrap();
    info!("Sent {}", wl);
}

fn main() {
    env_logger::init();

    let cmd = command::Command {
        msg_type: 0x10,
        len: 0,
        data: vec![0]
    };
    println!("{:?}", cmd);
    debug!("checksum {:?}", cmd.get_checksum());
    let msg = cmd.build();
    debug!("command {:?}", msg);

    let recver = thread::spawn(move || {
        recv();
    });
    let sender = thread::spawn(move || {
        thread::sleep(Duration::new(1,0));
        send(&msg);
    });
    sender.join().unwrap();
    recver.join().unwrap();
}
