use dns::{DNSMessage, DNSRecordType, QueryReply, ResourceRecord};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::thread::spawn;

pub mod dns;
pub mod rdt;

fn dns() {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 53)).unwrap();
    let mut buf = [0; 512]; // Ringrazio il limite di 512 byte, mi facilita il lavoro 😁!

    loop {
        let (_, src) = socket.recv_from(&mut buf).unwrap();
        let DNSMessage {
            identification,
            questions,
            ..
        } = DNSMessage::from(&buf);

        let response = DNSMessage {
            identification,
            query_reply: QueryReply::Reply,
            answers: vec![ResourceRecord {
                name: questions.first().unwrap().name.clone(),
                dns_record_type: DNSRecordType::A,
                class_code: questions[0].class_code,
                ttl: 300,
                rd_length: 4,
                r_data: vec![0, 0, 10, 8, 123, 5],
            }],
            ..Default::default()
        };

        let response: Vec<u8> = response.into();
        socket.send_to(&response, src).unwrap();
    }
}

fn rdt(src: SocketAddrV4, dest: SocketAddrV4) {
    let socket = UdpSocket::bind(src).unwrap();
    let mut buf = [0; 512]; // Ringrazio il limite di 512 byte, mi facilita il lavoro 😁!

    loop {
        let (_, src) = socket.recv_from(&mut buf).unwrap();

        let message: RDTRequest = RDTMessage::from(&buf);
        let response: RDTResponse = RDTResponse {
            sequence_number: message.sequence_number,
            ack_number: message.sequence_number + 1,
            payload: vec![0; 512],
        };

        socket.send_to(&response, src).unwrap();
    }
}

fn main() {
    // dns();
    let a = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3030);
    let b = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 4040);

    spawn(move || rdt(a, b));
    spawn(move || rdt(b, a));
}
