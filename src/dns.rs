pub enum QueryReply {
    Query,
    Reply,
}

impl Into<u16> for QueryReply {
    fn into(self) -> u16 {
        match self {
            Self::Query => 0,
            Self::Reply => 0x8000,
        }
    }
}

impl From<u16> for QueryReply {
    fn from(flags: u16) -> Self {
        match flags & 0x8000 {
            0x8000 => Self::Query,
            _ => Self::Reply,
        }
    }
}

#[non_exhaustive]
pub enum OpCode {
    Query,
    InverseQuery,
    Status,
    Notify,
    Update,
    DSO,
}

impl Into<u16> for OpCode {
    fn into(self) -> u16 {
        let code = match self {
            Self::Query => 0,
            Self::InverseQuery => 1,
            Self::Status => 2,
            Self::Notify => 4,
            Self::Update => 5,
            Self::DSO => 6,
        };

        code << 11
    }
}

impl From<u16> for OpCode {
    fn from(flags: u16) -> Self {
        match (flags & 0x7800) >> 11 {
            0 => Self::Query,
            1 => Self::InverseQuery,
            2 => Self::Status,
            4 => Self::Notify,
            5 => Self::Update,
            6 => Self::DSO,
            _ => unreachable!(),
        }
    }
}

#[non_exhaustive]
pub enum ResponseCode {
    NoError,
    FormErr,
    ServFail,
    NXDomain,
    NotImp,
    Refused,
}

impl Into<u16> for ResponseCode {
    fn into(self) -> u16 {
        match self {
            Self::NoError => 0,
            Self::FormErr => 1,
            Self::ServFail => 2,
            Self::NXDomain => 3,
            Self::NotImp => 4,
            Self::Refused => 5,
        }
    }
}

impl From<u16> for ResponseCode {
    fn from(value: u16) -> Self {
        match value & 0x000f {
            0 => Self::NoError,
            1 => Self::FormErr,
            2 => Self::ServFail,
            3 => Self::NXDomain,
            4 => Self::NotImp,
            5 => Self::Refused,
            _ => Self::ServFail,
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum DNSRecordType {
    A,
    NS,
    CNAME,
    MX,
    TXT,
    AAAA,
    ALL,
}

impl Into<u16> for DNSRecordType {
    fn into(self) -> u16 {
        match self {
            Self::A => 1,
            Self::NS => 2,
            Self::CNAME => 5,
            Self::MX => 15,
            Self::TXT => 16,
            Self::AAAA => 28,
            Self::ALL => 255,
        }
    }
}

impl From<u16> for DNSRecordType {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::A,
            2 => Self::NS,
            5 => Self::CNAME,
            15 => Self::MX,
            16 => Self::TXT,
            28 => Self::AAAA,
            _ => Self::A, // TODO: better solution
        }
    }
}

pub struct Question {
    pub name: Vec<Vec<u8>>,
    pub dns_type: DNSRecordType,
    pub class_code: u16,
}

pub struct ResourceRecord {
    pub name: Vec<Vec<u8>>,
    pub dns_record_type: DNSRecordType,
    pub class_code: u16,
    pub ttl: u32,
    pub rd_length: u16,
    pub r_data: Vec<u8>,
}

pub struct DNSMessage {
    pub identification: u16,
    pub query_reply: QueryReply,
    pub opcode: OpCode,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub response_code: ResponseCode,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authority_rrs: Vec<ResourceRecord>,
    pub additional_rrs: Vec<ResourceRecord>,
}

impl Default for DNSMessage {
    fn default() -> Self {
        DNSMessage {
            identification: 0,
            query_reply: QueryReply::Query,
            opcode: OpCode::Query,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: false,
            recursion_available: false,
            response_code: ResponseCode::NoError,
            questions: vec![],
            answers: vec![],
            authority_rrs: vec![],
            additional_rrs: vec![],
        }
    }
}

impl From<&[u8; 512]> for DNSMessage {
    fn from(buf: &[u8; 512]) -> Self {
        let identification = u16::from_be_bytes([buf[0], buf[1]]);
        let flags = u16::from_be_bytes([buf[2], buf[3]]);
        let number_of_questions = u16::from_be_bytes([buf[4], buf[5]]);

        let mut questions = vec![];
        let mut byte = 13;

        for _ in 0..number_of_questions {
            let mut name = vec![];

            loop {
                let len = buf[byte - 1] as usize;

                if len == 0 {
                    break;
                }

                name.push(buf[byte..byte + len].into());
                byte += len + 1;
            }

            questions.push(Question {
                name,
                dns_type: DNSRecordType::from(u16::from_be_bytes([buf[byte], buf[byte + 1]])),
                class_code: u16::from_be_bytes([buf[byte + 2], buf[byte + 3]]),
            });

            byte += 4;
        }

        Self {
            identification,
            query_reply: QueryReply::from(flags),
            opcode: OpCode::from(flags),
            authoritative_answer: flags & 0x0400 != 0,
            truncation: flags & 0x0200 != 0,
            recursion_desired: flags & 0x0100 != 0,
            recursion_available: flags & 0x0080 != 0,
            response_code: ResponseCode::from(flags),
            questions,
            ..Self::default()
        }
    }
}

impl Into<Vec<u8>> for DNSMessage {
    fn into(self) -> Vec<u8> {
        let flags = self.query_reply as u16
            | (self.opcode as u16)
            | (self.authoritative_answer as u16) << 10
            | (self.truncation as u16) << 9
            | (self.recursion_desired as u16) << 8
            | (self.recursion_available as u16) << 7
            | (self.response_code as u16);

        let mut response = vec![];

        response.extend(self.identification.to_be_bytes());
        response.extend(flags.to_be_bytes());
        response.extend((self.questions.len() as u16).to_be_bytes());
        response.extend((self.answers.len() as u16).to_be_bytes());
        response.extend((self.authority_rrs.len() as u16).to_be_bytes());
        response.extend((self.additional_rrs.len() as u16).to_be_bytes());

        for answer in self.answers.iter() {
            for subdomain in answer.name.iter() {
                response.push(subdomain.len() as u8);
                response.extend_from_slice(subdomain);
            }

            response.push(answer.dns_record_type as u8);
            response.extend(answer.class_code.to_be_bytes());
            response.extend(answer.ttl.to_be_bytes());
            response.extend(answer.rd_length.to_be_bytes());
            response.extend(&answer.r_data);
        }

        response
    }
}

