use std::path::Path;
use std::thread;
use std::time::Duration;
use byteorder::{BigEndian, WriteBytesExt};

use rppal::uart::{Parity, Uart};

#[derive(Debug)]
struct Command {
    msg_type: u16,
    len: u16,
    data: Vec<u8>,
}

impl Command {
    pub fn get_checksum(&self) -> u8 {
        let mut checksum = 0u8;

        checksum ^= (self.msg_type >> 8) as u8;
        checksum ^= (self.msg_type & 0xff) as u8;
        checksum ^= (self.len >> 8) as u8;
        checksum ^= (self.len & 0xff) as u8;

        for byte in &self.data {
            checksum ^= byte;
        }

        checksum
    }

    pub fn build(&self) -> Vec<u8> {
        let mut cmd = Vec::new();

        let mut hdr = vec![];
        hdr.write_u16::<BigEndian>(self.msg_type).unwrap();
        hdr.write_u16::<BigEndian>(self.len).unwrap();

        let transcoded_hdr = transcode(&hdr);

        let checksum = self.get_checksum();

        let transcoded_data = transcode(&self.data);

        cmd.push(0x01u8);
        for byte in transcoded_hdr {
            cmd.push(byte.to_owned());
        }
        cmd.push(checksum);
        for byte in transcoded_data {
            cmd.push(byte.to_owned());
        }
        cmd.push(0x03u8);

        cmd
    }
}

fn transcode(data: & Vec<u8>) -> Vec<u8> {
    let mut msg = Vec::new();

    for byte in data {
        if byte > &0x10 {
            msg.push(byte.to_owned());
        } else {
            msg.push(2u8);
            msg.push(byte.to_owned()^0x10);
        }
    }

    msg
}

fn recv() {
    let ama0 = Path::new("/dev/ttyAMA0");
    println!("Receiving from {}", ama0.display());

    let mut uart = Uart::with_path(&ama0, 115_200, Parity::None, 8, 1).unwrap();

    uart.set_read_mode(1, Duration::default()).unwrap();

    let mut buf = [0u8; 1];
    loop {
        if uart.read(&mut buf).unwrap() > 0 {
            println!("Received {}", buf[0]);
        }
    }
}

fn send(cmd: & Vec<u8>) {
    let ama0 = Path::new("/dev/ttyAMA0");
    println!("Send to {}", ama0.display());

    let mut uart = Uart::with_path(&ama0, 115_200, Parity::None, 8, 1).unwrap();

    uart.set_write_mode(true).unwrap();

    let wl = uart.write(cmd).unwrap();
    println!("Sent {}", wl);
}

fn main() {
    let cmd = Command {
        msg_type: 0x10,
        len: 0,
        data: vec![0]
    };
    println!("Command {:?}", cmd);
    println!("checksum {:?}", cmd.get_checksum());
    let msg = cmd.build();
    println!("command {:?}", msg);

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
