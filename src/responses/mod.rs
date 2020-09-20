use std::boxed::Box;
//use bytebuffer::ByteBuffer;
use num_traits::FromPrimitive;

use crate::command::{Command, MessageType};

mod status;
mod device_announce;
mod devices_list;
mod active_endpoints;

pub use status::Status;
pub use device_announce::DeviceAnnounce;
pub use devices_list::DevicesList;
pub use active_endpoints::ActiveEndpoints;

//enum ResponseType {
//    StatusResp(Status),
//    UnknownResp(UnknownResponse),
//}

//struct ResponseBox {
//    resp_type: ResponseType,
//    resp: &Box<dyn Response>,
//}

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
