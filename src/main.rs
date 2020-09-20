#[macro_use] extern crate log;

use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use rppal::uart::{Parity, Uart};
use hex::FromHex;

mod command;
mod responses;

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
                    Ok(cmd) => {
                        //println!("Received: {}", cmd);
                        let response = responses::parse_response(&cmd);
                        println!("{}", response.to_string());
                    },
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

fn start() {
    // Channel mask
    send_cmd(0x0021, vec![0, 0, 0, 11]);
    thread::sleep(Duration::new(1, 0));
    // Device type
    send_cmd(0x0023, vec![1]);
    thread::sleep(Duration::new(1, 0));
    // Network state
    send_cmd(0x0009, vec![]);
    thread::sleep(Duration::new(1, 0));
    // Version
    send_cmd(0x0010, vec![]);
}

fn start_inclusion() {
    // Permit join
    send_cmd(0x0049, vec![0xff, 0xfc, 0x1e, 0x00]);
}

fn devices_list() {
    send_cmd(0x0015, vec![]);
}

fn get_addr() -> Result<Vec<u8>, &'static str> {
    println!("enter device address: ");
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(5) => match Vec::from_hex(&buf[..buf.len()-1]) {
            Ok(address) => return Ok(address),
            _ => return Err("Decode error"),
        },
        _ => return Err("Wrong length"),
    };
}

fn endpoint_list() {
    match get_addr() {
        Ok(addr) => send_cmd(0x0045, addr),
        Err(msg) => println!(msg),
    }
}

fn list_commands() {
    println!("Commands:");
    println!("  help");
    println!("  start");
    println!("  start inclusion");
    println!("  devices list");
    println!("  endpoint list");
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
                    "help" => list_commands(),
                    "start" => start(),
                    "start inclusion" => start_inclusion(),
                    "devices list" => devices_list(),
                    "endpoint list" => endpoint_list(),
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
