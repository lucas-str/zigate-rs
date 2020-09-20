use bytebuffer::ByteBuffer;

use crate::responses::Response;
use crate::command::Command;

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
