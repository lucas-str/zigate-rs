#[macro_use] extern crate log;

use std::io;
use std::path::Path;
//use std::thread;
//use std::time::Duration;
//use std::sync::{Arc, Mutex};

// Hex parsing
//use std::io::Cursor;
//use byteorder::{BigEndian, ReadBytesExt};
//use hex::FromHex;

mod command;
mod commands;
mod responses;
mod serial;
mod zigate;

fn sender(mut zigate: zigate::Zigate) {
    loop {
        let mut buf = String::new();
        match io::stdin().read_line(&mut buf) {
            Ok(_n) => {
                let input = &buf[..buf.len()-1];
                match input {
                    "" => {},
                    "version" => zigate.send(&commands::get_version()),
                    unk => {
                        println!("Unknown command {}", unk);
                    },
                }
            }
            Err(error) => println!("Error: {}", error),
        }
    }
}

fn main() {
    env_logger::init();

    let path = Path::new("/dev/ttyAMA0");

    let mut zigate = zigate::Zigate::new(&path);

    let zhandle = zigate.start();

    //zigate.send(&commands::get_version());

    //thread::sleep(Duration::new(1, 0));
    match zigate.get_version() {
        Ok(version) => println!("{}", version),
        _ => println!("error"),
    }

    let devices = zigate.get_devices();
    for (addr, device) in devices {
        println!("DEVICES {:?}", device);
        let level = zigate.get_level(addr, device.endpoints[0].id);
        match level {
            Some(level) => println!("LEVEL {}", level),
            None => println!("FAILED TO READ LEVEL"),
        }
    }


    zhandle.join().unwrap();

    //let mut zigate = serial::UartSender::new(&path);
    //let rx = serial::uart_recver(&path);
    //let recv = thread::spawn(move || {
    //    loop {
    //        match rx.recv() {
    //            Ok(cmd) => println!("received {}", cmd),
    //            Err(err) => println!("error {}", err),
    //        }
    //    }
    //});

    //sender(zigate);

    //recv.join().unwrap();
}
