use bytebuffer::ByteBuffer;

use crate::responses::Response;
use crate::command::Command;

#[derive(Debug)]
pub struct SimpleDescriptorResponse {
    seq_num: u8,
    status: u8,
    addr: u16,
    len: u8,
    endpoint: u8,
    profile: u16,
    device_id: u16,
    version: u8, // 4 bits
    reserved: u8, // 4 bits
    in_cluster_count: u8,
    in_cluster_list: Vec<u16>,
    out_cluster_count: u8,
    out_cluster_list: Vec<u16>,
}

impl Response for SimpleDescriptorResponse {
    fn from_command(cmd: &Command) -> Result<SimpleDescriptorResponse, &'static str> {
        let mut buf = ByteBuffer::from_bytes(&cmd.data);

        let seq_num = buf.read_u8().unwrap();
        let status = buf.read_u8().unwrap();
        let addr = buf.read_u16().unwrap();
        let len = buf.read_u8().unwrap();
        let endpoint = buf.read_u8().unwrap();
        let profile = buf.read_u16().unwrap();
        let device_id = buf.read_u16().unwrap();
        let flags = buf.read_u8().unwrap();
        let version = flags >> 4;
        let reserved = flags & 0x0f;
        let in_cluster_count = buf.read_u8().unwrap();
        let mut in_cluster_list = Vec::new();
        for _ in 0..in_cluster_count {
            in_cluster_list.push(buf.read_u16().unwrap());
        }
        let out_cluster_count = buf.read_u8().unwrap();
        let mut out_cluster_list = Vec::new();
        for _ in 0..out_cluster_count {
            out_cluster_list.push(buf.read_u16().unwrap());
        }

        Ok(SimpleDescriptorResponse {
            seq_num,
            status,
            addr,
            len,
            endpoint,
            profile,
            device_id,
            version,
            reserved,
            in_cluster_count,
            in_cluster_list,
            out_cluster_count,
            out_cluster_list,
        })
    }
    fn to_string(&self) -> String {
        let mut s = String::from("Simple Descriptor Response: ");
        s.push_str(&format!("seq_num {} ", &self.seq_num));
        s.push_str(&format!("status {} ", &self.status));
        s.push_str(&format!("addr {:X} ", &self.addr));
        s.push_str(&format!("len {} ", &self.len));
        s.push_str(&format!("endpoint {} ", &self.endpoint));
        s.push_str(&format!("profile {} ", &self.profile));
        s.push_str(&format!("device id {} ", &self.device_id));
        s.push_str(&format!("device version {} ", &self.version));
        s.push_str(&format!("reserved {:#X} ", &self.reserved));
        s.push_str(&format!("in clusters {:?} ", &self.in_cluster_list));
        s.push_str(&format!("out clusters {:?} ", &self.out_cluster_list));
        s
    }
}
