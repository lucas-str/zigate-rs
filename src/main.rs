#[macro_use] extern crate log;

use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

// Hex parsing
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use hex::FromHex;

mod command;
mod commands;
mod responses;
mod zigate;

fn start(zigate: &mut zigate::Zigate) {
    zigate.send(&commands::set_channel_mask(11));
    thread::sleep(Duration::new(1, 0));
    zigate.send(&commands::set_device_type(commands::DeviceType::Coordinator));
    thread::sleep(Duration::new(1, 0));
    zigate.send(&commands::get_network_state());
    thread::sleep(Duration::new(1, 0));
    zigate.send(&commands::get_version());
}

fn start_inclusion(zigate: &mut zigate::Zigate) {
    zigate.send(&commands::permit_join_request(0xfffc, 0x1e, 0));
}

fn devices_list(zigate: &mut zigate::Zigate) {
    zigate.send(&commands::get_devices_list());
}

fn get_addr() -> Result<u16, &'static str> {
    println!("enter device address: ");
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(5) => match Vec::from_hex(&buf[..buf.len()-1]) {
            //Ok(address) => return Ok(address),
            Ok(address) => {
                let mut rdr = Cursor::new(address);
                return Ok(rdr.read_u16::<BigEndian>().unwrap())
            },
            _ => return Err("Decode error"),
        },
        _ => return Err("Wrong length"),
    };
}

fn endpoint_list(zigate: &mut zigate::Zigate) {
    match get_addr() {
        Ok(addr) => zigate.send(&commands::active_endpoint_request(addr)),
        Err(msg) => println!("{}", msg),
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

fn sender(zigate: &mut zigate::Zigate) {
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
                    "start" => start(zigate),
                    "start inclusion" => start_inclusion(zigate),
                    "devices list" => devices_list(zigate),
                    "endpoint list" => endpoint_list(zigate),
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

fn response_callback(cmd: &command::Command) {
    let response = responses::parse_response(cmd);
    println!("{}", response.to_string());
}

fn main() {
    env_logger::init();

    let path = Path::new("/dev/ttyAMA0");
    let mut zigate = zigate::Zigate::new(path);
    zigate.set_response_callback(response_callback);
    let handle = zigate.start().unwrap();

    sender(&mut zigate);

    handle.join().unwrap();
}
