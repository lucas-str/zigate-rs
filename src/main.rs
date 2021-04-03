#[macro_use] extern crate log;

use std::io;
use std::path::Path;
use std::{
    thread,
    time::Duration,
};

mod cluster;
mod command;
mod commands;
mod device;
mod responses;
mod serial;
mod zigate;

fn main() {
    env_logger::init();

    let path = Path::new("/dev/ttyAMA0");

    let mut zigate = zigate::Zigate::new(&path);

    let zhandle = zigate.start();

    //thread::sleep(Duration::new(1, 0));
    match zigate.get_version() {
        Ok(version) => println!("{}", version),
        _ => println!("error"),
    }

    let devices = zigate.get_devices();
    for (addr, device) in devices {
        println!("DEVICES {:?}", device);
        let on = zigate.get_onoff(addr, device.endpoints[0].id);
        match on {
            Ok(on) => println!("ON {}", on),
            _ => println!("FAILED TO ON/OFF"),
        }
        let level = zigate.get_level(addr, device.endpoints[0].id);
        match level {
            Ok(level) => println!("LEVEL {}", level),
            _ => println!("FAILED TO READ LEVEL"),
        }
        zigate.move_to_color_temp(addr, 3, 370, 10);
        thread::sleep(Duration::new(1, 0));
        let color_temp = zigate.get_color_temp(addr, device.endpoints[0].id);
        match color_temp {
            Ok(color_temp) => println!("COLOR TEMP {}", color_temp),
            _ => println!("FAILED TO READ COLOR TEMP"),
        }
    }

    thread::sleep(Duration::new(1, 0));
    //zhandle.join().unwrap();
}
