#[macro_use] extern crate log;

use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use rppal::uart::{Parity, Uart};

mod command;

fn recv() {
    let ama0 = Path::new("/dev/ttyAMA0");
    debug!("Receiving from {}", ama0.display());

    let mut uart = Uart::with_path(&ama0, 115_200, Parity::None, 8, 1).unwrap();

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
                    Ok(cmd) => println!("Received: {}", cmd),
                    Err(msg) => println!("Error: {}", msg),
                }
            }
        }
    }
}

fn send(cmd: & Vec<u8>) {
    let ama0 = Path::new("/dev/ttyAMA0");
    debug!("Send to {}", ama0.display());

    let mut uart = Uart::with_path(&ama0, 115_200, Parity::None, 8, 1).unwrap();

    uart.set_write_mode(true).unwrap();

    let wl = uart.write(cmd).unwrap();
    debug!("Sent {} bytes", wl);
}

fn send_cmd(msg_type: u16, data: Vec<u8>) {
    let cmd = command::Command::new(msg_type, data).unwrap();
    let msg = cmd.serialize();
    println!("Send: {}", cmd);
    send(&msg);
}

fn list_commands() {
    println!("Commands:");
    println!("  h: list commands");
    println!("  version");
    println!("  set mask: set mask to 00 00 08 00");
    println!("  start network");
    println!("  scan");
    println!("  reset");
    println!("  permit join");
    println!("  set permit: permit join for 10s");
    println!("  erase");
}

fn sender() {
    list_commands();
    loop {
        let mut buf = String::new();
        match io::stdin().read_line(&mut buf) {
            Ok(_n) => {
                let input = &buf[..buf.len()-1];
                match input {
                    "" => {},
                    "h" => list_commands(),
                    "version" => send_cmd(0x0010, vec![]),
                    "set mask" => send_cmd(0x0021, vec![0, 0, 8, 0]),
                    "start network" => send_cmd(0x0024, vec![]),
                    "scan" => send_cmd(0x0025, vec![]),
                    "reset" => send_cmd(0x0011, vec![]),
                    "permit join" => send_cmd(0x0014, vec![]),
                    "set permit" => send_cmd(0x0049, vec![0xff, 0xfc, 0x0a, 0x00]),
                    "erase" => send_cmd(0x0012, vec![]),
                    unk => {
                        println!("Unknown command {}", unk);
                        list_commands();
                    },
                }
            }
            Err(error) => println!("Error: {}", error),
        }
    }
}

fn main() {
    env_logger::init();

    let recver = thread::spawn(move || {
        recv();
    });

    let cmd = command::Command::new(0x0010, vec![]).unwrap();
    let msg = cmd.serialize();
    debug!("msg {:?}", msg);

    sender();

    recver.join().unwrap();
}
