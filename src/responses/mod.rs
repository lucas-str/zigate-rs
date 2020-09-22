//use std::boxed::Box;
//use bytebuffer::ByteBuffer;
use num_traits::FromPrimitive;

use crate::command::{Command, MessageType};

macro_rules! make_response_box {
    ( $($mod:ident, $box:ident ($resp:ident) ),+ ) => {
        $(mod $mod;)+
        $(pub use $mod::$resp;)+

        pub enum ResponseBox {
            $( $box($resp), )+
            UnknownBox(Unknown),
        }

        impl ResponseBox {
            pub fn to_string(&self) -> String {
                match self {
                    $( ResponseBox::$box(response) => response.to_string(), )+
                    ResponseBox::UnknownBox(response) => response.to_string(),
                }
            }
            pub fn from_command(cmd: &Command) -> ResponseBox {
                match FromPrimitive::from_u16(cmd.msg_type) {
                    $( Some(MessageType::$resp) => ResponseBox::$box($resp::from_command(&cmd).unwrap()), )+
                    Some(_) => ResponseBox::UnknownBox(Unknown::from_command(&cmd).unwrap()),
                    None => ResponseBox::UnknownBox(Unknown::from_command(&cmd).unwrap()),
                }
            }
        }
    }
}

make_response_box!(
    status, StatusBox(Status),
    device_announce, DeviceAnnounceBox(DeviceAnnounce),
    devices_list, DevicesListBox(DevicesList),
    active_endpoints, ActiveEndpointsBox(ActiveEndpoints),
    simple_descriptor, SimpleDescriptorResponseBox(SimpleDescriptorResponse)
    );

pub trait Response {
    fn from_command(cmd: &Command) -> Result<Self, &'static str> where Self: std::marker::Sized;
    fn to_string(&self) -> String;
}

#[derive(Debug)]
pub struct Unknown {
    msg_type: u16,
    data: Vec<u8>,
}

impl Response for Unknown {
    fn from_command(cmd: &Command) -> Result<Unknown, &'static str> {
        Ok(Unknown { msg_type: cmd.msg_type, data: cmd.data.clone() })
    }
    fn to_string(&self) -> String {
        String::from(
            format!("Unknown Response : type {:#X}, data {:X?}", self.msg_type, self.data))
    }
}
