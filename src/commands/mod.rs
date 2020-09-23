use byteorder::{BigEndian, WriteBytesExt};

use crate::command::{Command, MessageType};

pub fn get_network_state() -> Command {
    Command::new(MessageType::GetNetworkState as u16, vec![]).unwrap()
}

pub fn get_version() -> Command {
    Command::new(MessageType::GetVersion as u16, vec![]).unwrap()
}

pub fn reset() -> Command {
    Command::new(MessageType::Reset as u16, vec![]).unwrap()
}

pub fn erase() -> Command {
    Command::new(MessageType::Erase as u16, vec![]).unwrap()
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

pub fn start_network() -> Command {
    Command::new(MessageType::StartNetwork as u16, vec![]).unwrap()
}

pub fn simple_descriptor_request(addr: u16, endpoint: u8) -> Command {
    let mut data = vec![];
    data.write_u16::<BigEndian>(addr).unwrap();
    data.push(endpoint);
    Command::new(MessageType::SimpleDescriptorRequest as u16, data).unwrap()
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

pub fn action_move(addr: u16, src_endpoint: u8, dst_endpoint: u8, cmd: u8, mode: u8, rate: u8) ->
                   Command {
    let mut data = vec![];
    data.push(2); // short address mode
    data.write_u16::<BigEndian>(addr).unwrap();
    data.push(src_endpoint);
    data.push(dst_endpoint);
    data.push(cmd);
    data.push(mode); // 0 up, 1 down
    data.push(rate);
    Command::new(MessageType::ActionMove as u16, data).unwrap()
}

pub fn action_move_onoff(addr: u16, src_endpoint: u8, dst_endpoint: u8, cmd: u8, level: u8,
                         transition_time: u16) -> Command {
    let mut data = vec![];
    data.push(2); // short address mode
    data.write_u16::<BigEndian>(addr).unwrap();
    data.push(src_endpoint);
    data.push(dst_endpoint);
    data.push(cmd);
    data.push(level);
    data.write_u16::<BigEndian>(transition_time).unwrap();
    Command::new(MessageType::ActionMoveOnOff as u16, data).unwrap()
}

pub fn action_onoff(addr: u16, src_endpoint: u8, dst_endpoint: u8, cmd: u8) -> Command {
    let mut data = vec![];
    data.push(2); // short address mode
    data.write_u16::<BigEndian>(addr).unwrap();
    data.push(src_endpoint);
    data.push(dst_endpoint);
    data.push(cmd);
    Command::new(MessageType::ActionOnOff as u16, data).unwrap()
}

pub fn action_onoff_timed(addr: u16, src_endpoint: u8, dst_endpoint: u8, cmd: u8, on_time: u16,
                          off_time: u16) -> Command {
    let mut data = vec![];
    data.push(2); // short address mode
    data.write_u16::<BigEndian>(addr).unwrap();
    data.push(src_endpoint);
    data.push(dst_endpoint);
    data.push(cmd);
    data.write_u16::<BigEndian>(on_time).unwrap();
    data.write_u16::<BigEndian>(off_time).unwrap();
    Command::new(MessageType::ActionOnOffTimed as u16, data).unwrap()
}

pub fn action_onoff_effect(addr: u16, src_endpoint: u8, dst_endpoint: u8, cmd: u8, effect_id: u8,
                           effect_gradient: u8) -> Command {
    let mut data = vec![];
    data.push(2); // short address mode
    data.write_u16::<BigEndian>(addr).unwrap();
    data.push(src_endpoint);
    data.push(dst_endpoint);
    data.push(cmd);
    data.push(effect_id);
    data.push(effect_gradient);
    Command::new(MessageType::ActionOnOffEffect as u16, data).unwrap()
}
