use bytebuffer::ByteBuffer;

use crate::responses::Response;
use crate::command::Command;

#[derive(Debug)]
pub struct VersionList {
    pub major: u16,
    pub installer: u16,
}

impl Response for VersionList {
    fn from_command(cmd: &Command) -> Result<Self, &'static str> {
        let mut buf = ByteBuffer::from_bytes(&cmd.data);

        let major = buf.read_u16().unwrap();
        let installer = buf.read_u16().unwrap();
        Ok(Self { major, installer })
    }
    fn to_string(&self) -> String {
        String::from(
            format!("Version : major {}, installer {}", self.major, self.installer))
    }
}
