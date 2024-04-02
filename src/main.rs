use dns::{DNSMessage, DNSRecordType, QueryReply, ResourceRecord};
use rdt::{Number, RDT};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::thread::{self, spawn};
use std::time::Duration;

use crate::rdt::Checksum;

pub mod dns;
pub mod rdt;
pub mod udt;

pub fn dns() {
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

fn rdt_server(socket_addr: SocketAddrV4) {
    let socket = UdpSocket::bind(socket_addr).unwrap();
    let mut buf = [0; 512];

    loop {
        let (_, src) = socket.recv_from(&mut buf).unwrap();

        let message = RDT::from(&buf);
        println!("Server - Received {:?}", message);

        match message {
            RDT::Message { number, .. } => {
                let ack = RDT::Ack {
                    number,
                    checksum: Checksum::Ok,
                };

                println!("Server - Sending {:?}", ack);

                let mut ack: Vec<u8> = ack.into();
                udt::send(&socket, &mut ack, src);
            }
            _ => {}
        }
    }
}

fn rdt_client(socket_addr: SocketAddrV4, server_addr: SocketAddrV4) {
    let socket = UdpSocket::bind(socket_addr).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();

    let mut buf = [0; 512];

    loop {
        let message = RDT::Message {
            number: Number::Zero,
            payload: "ciao".as_bytes().into(),
            checksum: Checksum::Ok,
        };
        println!("Client - Sending {:?}", message);

        let mut message: Vec<u8> = message.into();
        udt::send(&socket, &mut message, server_addr.into());
        thread::sleep(Duration::from_millis(1000));

        match socket.recv_from(&mut buf) {
            Ok(_) => {
                let response = RDT::from(&buf);
                println!("Client - Received {:?}", response)
            }
            _ => println!("Client - ACK Timeout, resending"),
        }
    }
}

fn main() {
    let a = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3030);
    let b = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 4040);

    let server = spawn(move || rdt_server(a));
    let client = spawn(move || rdt_client(b, a));

    server.join().unwrap();
    client.join().unwrap();
}
