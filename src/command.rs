use std::fmt;
use bytebuffer::ByteBuffer;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive, Debug)]
pub enum MessageType {
    GetNetworkState = 0x0009,
    GetVersion = 0x0010,
    GetDevicesList = 0x0015,
    SetChannelMask = 0x0021,
    SetDeviceType = 0x0023,
    ActiveEndpoint = 0x0045,
    PermitJoinRequest = 0x0049,

    ActionOnOff = 0x0092,

    Status = 0x8000,
    DevicesList = 0x8015,
    ActiveEndpoints = 0x8045,

    DeviceAnnounce = 0x004D,
    Unknown,
}

#[derive(Debug)]
pub struct Command {
    pub msg_type: u16,
    pub data: Vec<u8>,
}

impl Command {

    pub fn new(msg_type: u16, data: Vec<u8>) -> Result<Command, &'static str> {
        if data.len() > 255 {
            return Err("Data length must be < 255")
        }
        Ok( Command { msg_type: msg_type, data: data } )
    }

    pub fn from_raw(msg: & Vec<u8>) -> Result<Command, &'static str> {

        if msg.len() < 7 {
            return Err("Raw message length must be greater than 7")
        }
        if msg[0] != 1 || msg[msg.len() - 1] != 3 {
            return Err("Incomplete raw message")
        }

        debug!("raw: {:?}", msg);
        let msg = decode(&msg[1..msg.len()-1]);
        debug!("decoded: {:?}", msg);

        let mut buf = ByteBuffer::from_bytes(&msg);

        let msg_type = buf.read_u16().unwrap();
        debug!("msg_type: {:#X}", msg_type);
        let len = buf.read_u16().unwrap();
        debug!("len: {}", len);
        let checksum = buf.read_u8().unwrap();
        debug!("checksum: {}", checksum);

        if msg.len() - 5 != len.into() {
            debug!("msg.len() {} len {}", msg.len(), len);
            return Err("Wrong data length")
        }

        let data = buf.read_bytes(len.into()).unwrap(); 
        debug!("data: {:?}", data);

        let cmd = Command {
            msg_type,
            data,
        };

        if cmd.get_checksum() != checksum {
            warn!("Invalid checksum")
            //return Err("Invalid checksum")
        }

        Ok(cmd)
    }

    pub fn get_checksum(&self) -> u8 {
        let mut checksum = 0u8;

        checksum ^= (self.msg_type >> 8) as u8;
        checksum ^= (self.msg_type & 0xff) as u8;
        checksum ^= (self.data.len() >> 8) as u8;
        checksum ^= (self.data.len() & 0xff) as u8;

        for byte in &self.data {
            checksum ^= byte;
        }

        checksum
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = ByteBuffer::new();
        buf.write_u16(self.msg_type);
        buf.write_u16(self.data.len() as u16);
        buf.write_u8(self.get_checksum());
        buf.write_bytes(&self.data);

        debug!("msg {:?}", buf.to_bytes());
        let mut msg = transcode(&buf.to_bytes());
        debug!("transcoded {:?}", msg);

        msg.insert(0, 1);
        msg.push(3);

        msg
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg_type = match FromPrimitive::from_u16(self.msg_type) {
            Some(msg_type) => msg_type,
            None => MessageType::Unknown,
        };
        write!(f, "{{{:?} ({:#X}), data: {:X?}}}", msg_type, self.msg_type, self.data)
    }
}

fn transcode(data: & Vec<u8>) -> Vec<u8> {
    let mut msg = Vec::new();

    for byte in data {
        if *byte > 0x10 {
            msg.push(*byte);
        } else {
            msg.push(2u8);
            msg.push(*byte^0x10);
        }
    }

    msg
}

fn decode(data: &[u8]) -> Vec<u8> {
    let mut msg = Vec::new();

    let mut transcode = false;

    for byte in data {
        if *byte == 2 {
            transcode = true;
        } else {
            if transcode {
                msg.push(*byte^0x10);
                transcode = false;
            } else {
                msg.push(*byte);
            }
        }
    }

    msg
}
