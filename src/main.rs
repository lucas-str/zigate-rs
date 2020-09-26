#[macro_use] extern crate log;

use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;
//use std::sync::{Arc, Mutex};

// Hex parsing
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use hex::FromHex;

mod command;
mod commands;
mod responses;
mod zigate;
mod serial;

fn start(zigate: &mut zigate::Zigate) {
    zigate.send(&commands::start_network());
    thread::sleep(Duration::new(2, 0));
    zigate.send(&commands::set_channel_mask(11));
    thread::sleep(Duration::new(2, 0));
    zigate.send(&commands::set_device_type(commands::DeviceType::Coordinator));
    thread::sleep(Duration::new(2, 0));
    zigate.send(&commands::get_network_state());
    thread::sleep(Duration::new(2, 0));
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

fn get_u8() -> Result<u8, &'static str> {
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(_) => match &buf[..buf.len()-1].parse::<u8>() {
            Ok(endpoint) => return Ok(*endpoint),
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

fn cluster_list(zigate: &mut zigate::Zigate) {
    match get_addr() {
        Ok(addr) => {
            println!("enter endpoint: ");
            match get_u8() {
                Ok(endpoint) => zigate.send(&commands::simple_descriptor_request(addr, endpoint)),
                Err(msg) => println!("{}", msg),
            }
        },
        Err(msg) => println!("{}", msg),
    }
}

fn action_onoff(zigate: &mut zigate::Zigate) {
    match get_addr() {
        Ok(addr) => {
            println!("enter endpoint: ");
            match get_u8() {
                Ok(endpoint) => {
                    println!("on/off (0/1): ");
                    match get_u8() {
                        Ok(c) => zigate.send(&commands::action_onoff(addr, 1, endpoint, c)),
                        Err(msg) => println!("{}", msg),
                    }
                },
                Err(msg) => println!("{}", msg),
            }
        },
        Err(msg) => println!("{}", msg),
    }
}

fn list_commands() {
    println!("Commands:");
    println!("  help");
    println!("  reset");
    println!("  erase");
    println!("  start");
    println!("  start inclusion");
    println!("  devices list");
    println!("  endpoint list");
    println!("  cluster list");
    println!("  action onoff");
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
                    "reset" => zigate.send(&commands::reset()),
                    "erase" => zigate.send(&commands::erase()),
                    "start" => start(zigate),
                    "start inclusion" => start_inclusion(zigate),
                    "devices list" => devices_list(zigate),
                    "endpoint list" => endpoint_list(zigate),
                    "cluster list" => cluster_list(zigate),
                    "action onoff" => action_onoff(zigate),
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
    let response = responses::ResponseBox::from_command(cmd);
    println!("{}", response.to_string());
    let mut zigate = zigate::Zigate::new(Path::new("/dev/ttyAMA0"));
    match response {
        responses::ResponseBox::DeviceAnnounceBox(device) => {
            println!("Got device announce, sending active endpoint request...");
            zigate.send(&commands::active_endpoint_request(device.short_address));
        },
        _ => (),
    }
}

fn response_callback2(cmd: &command::Command, _data: &mut u8) {
    let response = responses::ResponseBox::from_command(cmd);
    println!("{}", response.to_string());
    println!("{}", _data);
    let mut zigate = zigate::Zigate::new(Path::new("/dev/ttyAMA0"));
    match response {
        responses::ResponseBox::DeviceAnnounceBox(device) => {
            println!("Got device announce, sending active endpoint request...");
            zigate.send(&commands::active_endpoint_request(device.short_address));
        },
        _ => (),
    }
}

fn main() {
    env_logger::init();

    let path = Path::new("/dev/ttyAMA0");
    //let mut zigate = zigate::Zigate::new(path);
    //zigate.set_response_callback(response_callback);
    ////let handle = zigate.start().unwrap();
    ////let handle = zigate.start();

    //let mut data = Arc::new(Mutex::new(0u8));
    //let handle = zigate.start2(response_callback2, &data);
    //{
    //    let mut data = data.lock().unwrap();
    //    *data += 1;
    //}

    //sender(&mut zigate);

    //handle.join().unwrap();

    let mut sender = serial::UartSender::new(&path);
    let rx = serial::uart_recver(&path);
    sender.send(&commands::get_version());
    loop {
        match rx.recv() {
            Ok(cmd) => println!("received {}", cmd),
            Err(err) => println!("error {}", err),
        }
    }
}
