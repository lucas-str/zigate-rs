use bytebuffer::ByteBuffer;

use crate::responses::Response;
use crate::command::{Command, MessageType};

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
        let packet_type = MessageType::from_u16(self.packet_type);
        String::from(
            format!("Status : {} ({}), seq_num {}, packet_type {:?} ({:#X})", status, self.status,
            self.seq_num, packet_type, self.packet_type))
    }
}
