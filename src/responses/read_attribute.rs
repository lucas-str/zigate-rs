use bytebuffer::ByteBuffer;
use std::str;

use crate::command::Command;
use crate::responses::Response;

pub struct ReadAttributeResponse {
    pub seq_num: u8,
    pub src_addr: u16,
    pub endpoint: u8,
    pub cluster_id: u16,
    pub attr_enum: u16,
    pub attr_status: u8,
    pub attr_data_type: u8,
    pub attr_size: u16,
    pub data: Vec<u8>,
}

impl Response for ReadAttributeResponse {
    fn from_command(cmd: &Command) -> Result<Self, &'static str> {
        let mut buf = ByteBuffer::from_bytes(&cmd.data);
        let seq_num = buf.read_u8().unwrap();
        let src_addr = buf.read_u16().unwrap();
        let endpoint = buf.read_u8().unwrap();
        let cluster_id = buf.read_u16().unwrap();
        let attr_enum = buf.read_u16().unwrap();
        let attr_status = buf.read_u8().unwrap();
        let attr_data_type = buf.read_u8().unwrap();
        let attr_size = buf.read_u16().unwrap();
        let mut data = Vec::new();
        for _ in 0..attr_size {
            data.push(buf.read_u8().unwrap());
        }
        Ok(Self {
            seq_num,
            src_addr,
            endpoint,
            cluster_id,
            attr_enum,
            attr_status,
            attr_data_type,
            attr_size,
            data,
        })
    }
    fn to_string(&self) -> String {
        String::from(
            format!("Read Attribute Response : addr {:X}, endpoint {}, cluster {}, enum {}, status {}, attr enum {}, data type {},  data {:?}",
                    self.src_addr, self.endpoint, self.cluster_id, self.attr_enum,
                    self.attr_status, self.attr_enum, self.attr_data_type, self.data))
    }
}

impl ReadAttributeResponse {
    pub fn data_as_u8(&self) -> Result<u8, ()> {
        if self.data.len() < 1 {
            error!("Failed to read attribute as u8: not enough data.");
            return Err(());
        }
        Ok(self.data[0])
    }
    pub fn data_as_u16(&self) -> Result<u16, ()> {
        if self.data.len() < 2 {
            error!("Failed to read attribute as u16: not enough data.");
            return Err(());
        }
        Ok((self.data[0] as u16) << 8 | (self.data[1] as u16) & 0xff)
    }
    pub fn data_as_bool(&self) -> Result<bool, ()> {
        if self.data.len() < 1 {
            error!("Failed to read attribute as bool: not enough data.");
            return Err(());
        }
        Ok(self.data[0] != 0)
    }
    pub fn data_as_str(&self) -> Result<&str, str::Utf8Error> {
        let res = str::from_utf8(&self.data);
        if let Err(e) = res {
            error!("Invalid UTF-8: {}", e);
        }
        res
    }
}
