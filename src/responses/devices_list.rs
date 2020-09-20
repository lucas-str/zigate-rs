use bytebuffer::ByteBuffer;

use crate::responses::Response;
use crate::command::Command;

#[derive(Debug)]
struct Device {
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
