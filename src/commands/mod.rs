use byteorder::{BigEndian, WriteBytesExt};

use crate::command::{Command, MessageType};

pub fn get_network_state() -> Command {
    Command::new(MessageType::GetNetworkState as u16, vec![]).unwrap()
}

pub fn get_version() -> Command {
    Command::new(MessageType::GetVersion as u16, vec![]).unwrap()
}

pub fn get_devices_list() -> Command {
    Command::new(MessageType::GetDevicesList as u16, vec![]).unwrap()
}

pub fn set_channel_mask(mask: u32) -> Command {
    let mut data = vec![];
    data.write_u32::<BigEndian>(mask).unwrap();
    Command::new(MessageType::SetChannelMask as u16, data).unwrap()
}

#[allow(dead_code)]
pub enum DeviceType {
    Coordinator = 0,
    Router = 1,
}

pub fn set_device_type(device_type: DeviceType) -> Command {
    let data = vec![device_type as u8];
    Command::new(MessageType::SetDeviceType as u16, data).unwrap()
}

pub fn active_endpoint_request(addr: u16) -> Command {
    let mut data = vec![];
    data.write_u16::<BigEndian>(addr).unwrap();
    Command::new(MessageType::ActiveEndpoint as u16, data).unwrap()
}

pub fn permit_join_request(addr: u16, interval: u8, tc_significance: u8) -> Command {
    let mut data = vec![];
    data.write_u16::<BigEndian>(addr).unwrap();
    data.push(interval);
    data.push(tc_significance);
    Command::new(MessageType::PermitJoinRequest as u16, data).unwrap()
}
