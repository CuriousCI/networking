// pub enum Response {
//     Ack(u8),
// }
//
// pub enum Request {
//     Wait0,
//     WaitAck0,
//     Wait1,
//     WaitAck1,
// }

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

pub enum RDT {
    Ack(Number),
    Message((Number, Vec<u8>)),
}

impl Into<Vec<u8>> for RDT {
    fn into(self) -> Vec<u8> {
        match self {
            RDT::Ack(number) => vec![number as u8],
            RDT::Message((number, payload)) => {
                let mut buf = vec![0x80 | number as u8 | payload.len() as u8];
                buf.extend(payload);
                buf
            }
        }
    }
}

impl From<&[u8]> for RDT {
    fn from(buf: &[u8]) -> Self {
        match buf[0] & 0x80 {
            0 => RDT::Ack(Number::from(buf[0])),
            _ => {
                let number = Number::from(buf[0]);
                let payload = buf[1..].to_vec();
                RDT::Message((number, payload))
            }
        }
    }
}

pub enum Checksum {
    Ok,
    Error,
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

pub mod udt {
    use rand::{
        distributions::{Bernoulli, Distribution},
        thread_rng,
    };
    use std::net::{SocketAddrV4, UdpSocket};

    pub fn send(socket: UdpSocket, buf: &[u8], addr: SocketAddrV4) {
        let corrupted = Bernoulli::new(0.1).unwrap();
        let not_sent = Bernoulli::new(0.01).unwrap();

        if corrupted.sample(&mut thread_rng()) {
            println!("Packet corrupted");
        }

        if not_sent.sample(&mut thread_rng()) {
            println!("Packet not sent");
        }

        // let v = corruption.sample(&mut thread_rng());
        // println!("{} is from a Bernoulli distribution", v);

        // non è arrivato
        // è arrivato male

        socket.send_to(buf, addr).unwrap();
    }
}

