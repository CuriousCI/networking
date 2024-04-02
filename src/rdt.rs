#[derive(Debug)]
pub enum Checksum {
    Ok,
    Error,
    // None?
}

impl Into<u8> for Checksum {
    fn into(self) -> u8 {
        match self {
            Checksum::Ok => 0,
            Checksum::Error => 1,
        }
    }
}

impl From<u8> for Checksum {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Checksum::Ok,
            _ => Checksum::Error,
        }
    }
}

#[derive(Debug)]
pub enum Number {
    Zero,
    One,
}

impl Into<u8> for Number {
    fn into(self) -> u8 {
        match self {
            Number::Zero => 0,
            Number::One => 0x40,
        }
    }
}

impl From<u8> for Number {
    fn from(byte: u8) -> Self {
        match byte & 0x40 {
            0 => Number::Zero,
            _ => Number::One,
        }
    }
}

#[derive(Debug)]
pub enum RDT {
    Ack {
        number: Number,
        checksum: Checksum,
    },
    Message {
        number: Number,
        payload: Vec<u8>,
        checksum: Checksum,
    },
}

impl Into<Vec<u8>> for RDT {
    fn into(self) -> Vec<u8> {
        match self {
            RDT::Ack { number, checksum } => vec![number as u8, checksum as u8],
            RDT::Message {
                number,
                payload,
                checksum,
            } => {
                let mut buf = vec![0x80 | number as u8 | payload.len() as u8];
                buf.extend(payload);
                buf.push(checksum as u8);
                buf
            }
        }
    }
}

impl From<&[u8; 512]> for RDT {
    fn from(buf: &[u8; 512]) -> Self {
        match buf[0] & 0x80 {
            0 => RDT::Ack {
                number: Number::from(buf[0]),
                checksum: Checksum::from(buf[1]),
            },
            _ => {
                let number = Number::from(buf[0]);
                let len = (buf[0] & 0x3f) as usize;
                let payload = buf[1..len + 1].to_vec();
                let checksum = Checksum::from(buf[len + 1]);

                RDT::Message {
                    number,
                    payload,
                    checksum,
                }
            }
        }
    }
}
