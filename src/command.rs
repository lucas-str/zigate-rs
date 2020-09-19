use byteorder::{BigEndian, WriteBytesExt};

#[derive(Debug)]
pub struct Command {
    pub msg_type: u16,
    pub len: u16,
    pub data: Vec<u8>,
}

impl Command {

    pub fn from_raw(msg: & Vec<u8>) -> Result<Command, &'static str> {

        if msg.len() < 5 {
            return Err("Raw message length must be greater than 5")
        }

        debug!("raw : {:?}", msg);
        let msg = decode(&msg);
        debug!("transcoded : {:?}", msg);

        let msg_type = (msg[0] as u16) << 8 | (msg[1] as u16) & 0xff;
        let len = (msg[2] as u16) << 8 | (msg[3] as u16) & 0xff;
        let checksum = msg[4];

        if msg.len() - 5 != len.into() {
            return Err("Wrong data length")
        }

        let mut data = Vec::new();
        for byte in &msg[5..] {
            data.push(byte.to_owned());
        }

        let cmd = Command {
            msg_type,
            len,
            data,
        };

        if cmd.get_checksum() != checksum {
            return Err("Invalid checksum")
        }

        Ok(cmd)
    }

    pub fn get_checksum(&self) -> u8 {
        let mut checksum = 0u8;

        checksum ^= (self.msg_type >> 8) as u8;
        checksum ^= (self.msg_type & 0xff) as u8;
        checksum ^= (self.len >> 8) as u8;
        checksum ^= (self.len & 0xff) as u8;

        for byte in &self.data {
            checksum ^= byte;
        }

        checksum
    }

    pub fn build(&self) -> Vec<u8> {
        let mut cmd = Vec::new();

        /* Do not transcode checksum */
        //let mut hdr = vec![];
        //hdr.write_u16::<BigEndian>(self.msg_type).unwrap();
        //hdr.write_u16::<BigEndian>(self.len).unwrap();

        //let transcoded_hdr = transcode(&hdr);

        //let checksum = self.get_checksum();

        //let transcoded_data = transcode(&self.data);

        //cmd.push(0x01u8);
        //for byte in transcoded_hdr {
        //    cmd.push(byte.to_owned());
        //}
        //cmd.push(checksum);
        //for byte in transcoded_data {
        //    cmd.push(byte.to_owned());
        //}
        //cmd.push(0x03u8);

        //cmd

        /* Transcode checksum */
        cmd.write_u16::<BigEndian>(self.msg_type).unwrap();
        cmd.write_u16::<BigEndian>(self.len).unwrap();
        cmd.push(self.get_checksum());
        cmd.extend_from_slice(&self.data);

        let transcoded = transcode(&cmd);

        let mut msg = vec![1u8];
        msg.extend_from_slice(&transcoded);
        msg.push(3u8);

        msg
    }
}

fn transcode(data: & Vec<u8>) -> Vec<u8> {
    let mut msg = Vec::new();

    for byte in data {
        if byte > &0x10 {
            msg.push(byte.to_owned());
        } else {
            msg.push(2u8);
            msg.push(byte.to_owned()^0x10);
        }
    }

    msg
}

fn decode(data: & Vec<u8>) -> Vec<u8> {
    let mut msg = Vec::new();

    let mut transcode = false;

    for byte in data {
        if byte == &2 {
            transcode = true;
        } else {
            if transcode {
                msg.push(byte.to_owned()^0x10);
            } else {
                msg.push(byte.to_owned());
            }
        }
    }

    msg
}
