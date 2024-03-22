use std::net::{SocketAddr, UdpSocket};

// QR	Indicates if the message is a query (0) or a reply (1)	1
// OPCODE	The type can be QUERY (standard query, 0), IQUERY (inverse query, 1), or STATUS (server status request, 2)	4
// AA	Authoritative Answer, in a response, indicates if the DNS server is authoritative for the queried hostname	1
// TC	TrunCation, indicates that this message was truncated due to excessive length	1
// RD	Recursion Desired, indicates if the client means a recursive query	1
// RA	Recursion Available, in a response, indicates if the replying DNS server supports recursion	1
// Z	Zero, reserved for future use	3
// RCODE	Response code, can be NOERROR (0), FORMERR (1, Format error), SERVFAIL (2), NXDOMAIN (3, Nonexistent domain), etc.[37]	4

//                      1 1 1 1 1 1 1 1 1 1 2 2 2 2 2 2 2 2 2 2 3 3
//  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// | transaction id (2 bytes)      | flags (2 bytes)               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// | # of questions (2 bytes)      | # of answers (2 bytes)        |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// | # of authority RRs (2 bytes)  | # of additional RRs (2 bytes) |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                               |
// /                           questions                           /
// |                                                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                               |
// /                           answer RRs                          /
// |                                                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                               |
// /                         authority RRs                         /
// |                                                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                                                               |
// /                         additional RRs                        /
// |                                                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

//                      1 1 1 1 1 1
//  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                               |
// |                               |
// /             name              /
// |                               |
// |                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |             type              |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |             class             |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |              ttl              |
// |                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |            rdlength           |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// |                               |
// |                               |
// /             rdata             /
// /                               /
// |                               |
// |                               |
// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

enum QueryReply {
    Query,
    Reply,
}

impl Into<u8> for QueryReply {
    fn into(self) -> u8 {
        match self {
            Self::Query => 0,
            Self::Reply => 1,
        }
    }
}

enum OpCode {
    Query,
    InverseQuery,
    Status,
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            Self::Query => 0,
            Self::InverseQuery => 1,
            Self::Status => 2,
        }
    }
}

#[non_exhaustive]
enum Zero {}

#[non_exhaustive]
enum ResponseCode {
    NoError,
    FormatError,
    ServerFail,
    NonexistentDomain,
}

impl Into<u8> for ResponseCode {
    fn into(self) -> u8 {
        match self {
            Self::NoError => 0,
            Self::FormatError => 1,
            Self::ServerFail => 2,
            Self::NonexistentDomain => 3,
        }
    }
}

struct DNSMessage {
    identification: u16,
    query_reply: QueryReply,
    opcode: OpCode, // 4 bit
    authoritative_answer: bool,
    truncation: bool, // Perché DNS su UDP ha un limite di 512 byte
    recursion_desired: bool,
    recursion_available: bool,
    _zero: (),                   // 3 bit
    response_code: ResponseCode, // 4 bit
    number_of_questions: u16,
    number_of_answers: u16,
    number_of_authority_rrs: u16,
    number_of_additional_rrs: u16,
    questions: Box<[Question]>,
    answers: Box<[ResourceRecord]>,
    authority_rrs: Box<[ResourceRecord]>,
    additional_rrs: Box<[ResourceRecord]>,
}

// TODO: Enum for ClassCode

#[non_exhaustive]
#[derive(Clone, Copy)]
enum DNSRecordType {
    A,
    NS,
    CNAME,
    MX,
    TXT,
    AAAA,
}

// https://en.wikipedia.org/wiki/List_of_DNS_record_types
// https://www.iana.org/assignments/dns-parameters/dns-parameters.xhtml

impl Into<u16> for DNSRecordType {
    fn into(self) -> u16 {
        match self {
            Self::A => 1,
            Self::NS => 2,
            Self::CNAME => 5,
            Self::MX => 15,
            Self::TXT => 16,
            Self::AAAA => 28,
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
            _ => Self::A, // TODO: better solution, unreachable!() is wrong here
        }
    }
}

struct Question {
    name: Box<[u8]>,
    dns_type: DNSRecordType,
    class_code: u16,
}

struct ResourceRecord {
    name: Box<[u8]>,
    dns_type: DNSRecordType,
    class_code: u16,
    ttl: u32,
    rd_length: u16,
    r_data: Box<[u8]>,
}

// https://en.wikipedia.org/wiki/Domain_Name_System#:~:text=address%20changes%20administratively.-,DNS%20message%20format,content%20of%20these%20four%20sections.

impl Into<Box<[u8]>> for DNSMessage {
    fn into(self) -> Box<[u8]> {
        let [id_high, id_low] = self.identification.to_be_bytes();
        let [q_high, q_low] = self.number_of_questions.to_be_bytes();
        let [a_high, a_low] = self.number_of_answers.to_be_bytes();
        let [arrs_high, arrs_low] = self.number_of_authority_rrs.to_be_bytes();
        let [addrrs_high, addrrs_low] = self.number_of_additional_rrs.to_be_bytes();

        let flag_high = (self.query_reply as u8) << 7
            | (self.opcode as u8) << 3
            | (self.authoritative_answer as u8) << 2
            | (self.truncation as u8) << 1
            | self.recursion_desired as u8;
        let flag_low = (self.recursion_available as u8) << 7 | self.response_code as u8;

        let mut response = vec![
            id_high,
            id_low,
            flag_high,
            flag_low,
            q_high,
            q_low,
            a_high,
            a_low,
            arrs_high,
            arrs_low,
            addrrs_high,
            addrrs_low,
        ];

        for answer in self.answers.iter() {
            for slice in answer.name.clone().split(|&byte| byte == b'.') {
                response.push(slice.len() as u8);
                response.extend_from_slice(slice);
            }

            // https://www.rfc-editor.org/rfc/rfc6895.html
            response.extend_from_slice(&(answer.dns_type as u8).to_be_bytes());
            response.extend_from_slice(&answer.class_code.to_be_bytes());
            response.extend_from_slice(&answer.ttl.to_be_bytes());
            response.extend_from_slice(&answer.rd_length.to_be_bytes());
            response.extend_from_slice(&answer.r_data);
        }

        response.into_boxed_slice()
    }
}

impl From<Box<[u8]>> for DNSMessage {
    fn from(bytes: Box<[u8]>) -> Self {
        let identification = u16::from_be_bytes([bytes[0], bytes[1]]);
        let flags_high = bytes[2];
        let flags_low = bytes[3];
        let number_of_questions = u16::from_be_bytes([bytes[4], bytes[5]]);
        let number_of_answers = u16::from_be_bytes([bytes[6], bytes[7]]);
        let number_of_authority_rrs = u16::from_be_bytes([bytes[8], bytes[9]]);
        let number_of_additional_rrs = u16::from_be_bytes([bytes[10], bytes[11]]);

        let mut index = 12;
        let mut length = bytes[index];
        index += 1;
        let mut domain = vec![];
        let mut questions = vec![];

        for _ in 0..number_of_questions {
            while length > 0 {
                for _ in 0..length {
                    domain.push(bytes[index]);
                    index += 1;
                }
                domain.push(b'.');
                length = bytes[index];
                index += 1;
            }
            let dns_type =
                DNSRecordType::from(u16::from_be_bytes([bytes[index], bytes[index + 1]]));
            let class_code = u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]);
            domain.pop();

            questions.push(Question {
                name: domain.clone().into_boxed_slice(),
                dns_type,
                class_code,
            });
            domain.clear();
            index += 4;
        }

        // non le gestisco per ora...
        for _ in 0..number_of_answers {}

        for _ in 0..number_of_authority_rrs {}

        for _ in 0..number_of_additional_rrs {}

        Self {
            identification,
            query_reply: QueryReply::Query,
            opcode: OpCode::Query,
            authoritative_answer: flags_high & 0b00000100 != 0,
            truncation: flags_high & 0b00000010 != 0,
            recursion_desired: flags_high & 0b00000001 != 0,
            recursion_available: flags_low & 0b10000000 != 0,
            _zero: (),
            response_code: ResponseCode::NoError, // TODO
            number_of_questions,
            number_of_answers,
            number_of_authority_rrs,
            number_of_additional_rrs,
            questions: questions.into_boxed_slice(),
            answers: Box::new([]),
            authority_rrs: Box::new([]),
            additional_rrs: Box::new([]),
        }
    }
}

fn main() {
    let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 53))).unwrap();

    let mut buffer = [0; 512]; // Ringrazio il limite di 512 byte, mi facilita il lavoro 😁!
    loop {
        let (size, src) = socket.recv_from(&mut buffer).unwrap();
        let request = DNSMessage::from(Box::from(buffer));

        println!("Received {} bytes from {}", size, src);
        println!("Data {:?}", buffer);
        println!(
            "Request {:?} - {:?}",
            String::from_utf8_lossy(&request.questions[0].name),
            request.questions[0].dns_type as u8
        );

        // Nota interessante: se il campo identification è diverso da quello della richiesta,
        // nslookup crede dice che c'è un timeout (pensa che la risposta non sia stata ricevuta)
        let message = DNSMessage {
            identification: request.identification,
            query_reply: QueryReply::Reply,
            opcode: OpCode::Query,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: false,
            recursion_available: false,
            _zero: (),
            response_code: ResponseCode::NoError,
            number_of_questions: 0,
            number_of_answers: 1,
            number_of_authority_rrs: 0,
            number_of_additional_rrs: 0,
            questions: Box::new([]),
            answers: Box::new([ResourceRecord {
                name: request.questions[0].name.clone(),
                dns_type: DNSRecordType::A,
                class_code: request.questions[0].class_code,
                ttl: 10, // 10 secondi
                rd_length: 4,
                r_data: Box::new([0, 0, 10, 8, 123, 5]),
            }]),
            authority_rrs: Box::new([]),
            additional_rrs: Box::new([]),
        };
        let body: Box<[u8]> = message.into();
        println!("Response {:?}", body);
        socket.send_to(&body, src).unwrap();
    }
}

// let identification = u16::from_be_bytes([bytes[0], bytes[1]]);
// let flags = u16::from_be_bytes([bytes[2], bytes[3]]);
// let number_of_questions = u16::from_be_bytes([bytes[4], bytes[5]]);
// let number_of_answers = u16::from_be_bytes([bytes[6], bytes[7]]);
// let number_of_authority_rrs = u16::from_be_bytes([bytes[8], bytes[9]]);
// let number_of_additional_rrs = u16::from_be_bytes([bytes[10], bytes[11]]);

// identification: 0000 0010 // valore 2
// flags: 0001 0000 // solo recursion_desired
// # questions: 0000 0001 // un solo dominio richiesto
// # answers: 0000 0000 // non ha risposte
// # authority RRs: 0000 0000 // non ha risposte dall'autorità
// # additional RRs: 0000 0000 // non ha riposte aggiuntive
// I 6 caratteri successivi indicano parte di un dominio
// google: 103, 111, 111, 103, 108, 101
// I 3 caratteri successivi indicano parte di un dominio
// com: 99, 111, 109
// I 5 caratteri successivi indicano parte di un dominio
// wind3: 119, 105, 110, 100, 51
// I 3 caratteri successivi indicano parte di un dominio
// hub: 104, 117, 98
// I 0 caratteri successivi indicano fine del dominio
// dns_type: 0000 0001 // A
// class_code: 0000 0001

// [0, 2, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 5, 119, 105, 110, 100, 51, 3, 104, 117, 98, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, ..., 0]
// [0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11, ..., 512]
