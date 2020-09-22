use bytebuffer::ByteBuffer;

use crate::responses::Response;
use crate::command::Command;

#[derive(Debug)]
pub struct DeviceAnnounce {
    pub short_address: u16,
    pub ieee_address: u64,
    pub mac_capability: u8,
    pub flags: u8,
    pub rejoin_info: u8,
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
