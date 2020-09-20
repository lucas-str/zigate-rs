use std::boxed::Box;
use bytebuffer::ByteBuffer;
use num_traits::FromPrimitive;

use crate::command::{Command, MessageType};

pub fn parse_response(cmd: &Command) -> Box<dyn Response> {
    match FromPrimitive::from_u16(cmd.msg_type) {
        Some(MessageType::Status) => Box::new(Status::from_command(&cmd).unwrap()),
        Some(MessageType::DeviceAnnounce) => Box::new(DeviceAnnounce::from_command(&cmd).unwrap()),
        Some(MessageType::DevicesList) => Box::new(DevicesList::from_command(&cmd).unwrap()),
        Some(MessageType::ActiveEndpoints) => Box::new(ActiveEndpoints::from_command(&cmd).unwrap()),
        Some(_) => Box::new(UnknownResponse::from_command(&cmd).unwrap()),
        None => Box::new(UnknownResponse::from_command(&cmd).unwrap()),
    }
}

pub trait Response {
    fn from_command(cmd: &Command) -> Result<Self, &'static str> where Self: std::marker::Sized;
    fn to_string(&self) -> String;
}

#[derive(Debug)]
pub struct UnknownResponse {
    msg_type: u16,
    data: Vec<u8>,
}

impl Response for UnknownResponse {
    fn from_command(cmd: &Command) -> Result<UnknownResponse, &'static str> {
        Ok(UnknownResponse { msg_type: cmd.msg_type, data: cmd.data.clone() })
    }
    fn to_string(&self) -> String {
        String::from(
            format!("Unknown Response : type {:#X}, data {:X?}", self.msg_type, self.data))
    }
}

#[derive(Debug)]
pub struct Status {
    status: u8,
    seq_num: u8,
    packet_type: u16,
}

impl Response for Status {
    fn from_command(cmd: &Command) -> Result<Status, &'static str> {
        let mut buf = ByteBuffer::from_bytes(&cmd.data);
        let status = match buf.read_u8() {
            Ok(status) => status,
            Err(_) => return Err("Failed to read status"),
        };
        let seq_num = match buf.read_u8() {
            Ok(seq_num) => seq_num,
            Err(_) => return Err("Failed to read sequence number"),
        };
        let packet_type = match buf.read_u16() {
            Ok(packet_type) => packet_type,
            Err(_) => return Err("Failed to read packet_type"),
        };

        Ok(Status { status, seq_num, packet_type })
    }
    fn to_string(&self) -> String {
        let status = match self.status {
            0 => "Success",
            1 => "Incorrect parameters",
            2 => "Unhandled command",
            3 => "Command failed",
            4 => "Busy",
            5 => "Stack already started",
            _ => "Failed",
        };
        String::from(
            format!("Status : {}, seq_num {}, packet_type {:#X}", status, self.seq_num,
                    self.packet_type))
    }
}

#[derive(Debug)]
pub struct DeviceAnnounce {
    short_address: u16,
    ieee_address: u64,
    mac_capability: u8,
    flags: u8,
    rejoin_info: u8,
}

impl Response for DeviceAnnounce {
    fn from_command(cmd: &Command) -> Result<DeviceAnnounce, &'static str> {
        let mut buf = ByteBuffer::from_bytes(&cmd.data);
        let short_address = match buf.read_u16() {
            Ok(short_address) => short_address,
            Err(_) => return Err("Failed to read short address"),
        };
        let ieee_address = match buf.read_u64() {
            Ok(ieee_address) => ieee_address,
            Err(_) => return Err("Failed to read IEEE address"),
        };
        let mac_capability = match buf.read_u8() {
            Ok(mac_capability) => mac_capability,
            Err(_) => return Err("Failed to read MAC capability"),
        };
        let flags = match buf.read_u8() {
            Ok(flags) => flags,
            Err(_) => return Err("Failed to read flags"),
        };
        let rejoin_info = match buf.read_u8() {
            Ok(rejoin_info) => rejoin_info,
            Err(_) => 0,
        };

        Ok(DeviceAnnounce { short_address, ieee_address, mac_capability, flags, rejoin_info })
    }
    fn to_string(&self) -> String {
        String::from(format!(
                "DeviceAnnounce : short address {:X}, IEEE address {:X}, MAC capability {}, flags {:b}, rejoin info {}",
                self.short_address, 
                self.ieee_address,
                self.mac_capability,
                self.flags,
                self.rejoin_info))
    }
}

#[derive(Debug)]
pub struct Device {
    id: u8,
    short_address: u16,
    ieee_address: u64,
    power_source: bool,
    link_quality: u8,
}

impl Device {
    fn to_string(&self) -> String {
        let power_source = match self.power_source {
            true => "power",
            false => "battery",
        };
        String::from(format!(
                "{{ id {}, short address {:X}, IEEE address {:X}, power source {}, link quality {} }}",
                self.id,
                self.short_address,
                self.ieee_address,
                power_source,
                self.link_quality))
    }
}

#[derive(Debug)]
pub struct DevicesList {
    devices: Vec<Device>,
}

impl Response for DevicesList {
    fn from_command(cmd: &Command) -> Result<DevicesList, &'static str> {
        let mut buf = ByteBuffer::from_bytes(&cmd.data);

        let mut remaining_len = cmd.data.len();
        let mut devices = vec![];
        while remaining_len >= 13 {
            let id = buf.read_u8().unwrap();
            let short_address = buf.read_u16().unwrap();
            let ieee_address = buf.read_u64().unwrap();
            let power_source = buf.read_u8().unwrap() == 1;
            let link_quality = buf.read_u8().unwrap();
            let device = Device { id, short_address, ieee_address, power_source, link_quality };
            devices.push(device);
            remaining_len -= 13;
        }

        if remaining_len != 0 {
            let remaining = cmd.data.len() - remaining_len;
            warn!("Remaining data while parsing DevicesList: {:X?}", &cmd.data[remaining..]);
        }

        Ok( DevicesList { devices } )
    }
    fn to_string(&self) -> String {
        let mut devices = String::from("DevicesList: [");
        for device in &self.devices {
            devices.push_str(&format!("\n  {},", device.to_string()));
        }
        devices.push_str(" ]");
        devices
    }
}

#[derive(Debug)]
pub struct ActiveEndpoints {
    seq_num: u8,
    status: u8,
    address: u16,
    endpoint_count: u8,
    endpoint_list: Vec<u8>,
}

impl Response for ActiveEndpoints {
    fn from_command(cmd: &Command) -> Result<ActiveEndpoints, &'static str> {
        let mut buf = ByteBuffer::from_bytes(&cmd.data);
        if cmd.data.len() < 5 {
            return Err("Not enough data")
        }

        let seq_num = buf.read_u8().unwrap();
        let status = buf.read_u8().unwrap();
        let address = buf.read_u16().unwrap();
        let endpoint_count = buf.read_u8().unwrap();

        let mut endpoint_list = Vec::new();
        for _ in 0..endpoint_count {
            match buf.read_u8() {
                Ok(endpoint) => endpoint_list.push(endpoint),
                Err(_) => return Err("Failed to get endpoint"),
            }
        }

        Ok( ActiveEndpoints { seq_num, status, address, endpoint_count, endpoint_list } )
    }
    fn to_string(&self) -> String {
        String::from(format!(
                "Active Endpoint : seq_num {}, status {}, address {:X}, endpoint count {}, endpoints {:?}",
                self.seq_num,
                self.status,
                self.address,
                self.endpoint_count,
                self.endpoint_list))
    }
}

