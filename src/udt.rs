use rand::{
    distributions::{Bernoulli, Distribution},
    thread_rng,
};
use std::net::{SocketAddr, UdpSocket};

pub fn send(socket: &UdpSocket, buf: &mut [u8], addr: SocketAddr) {
    let corrupted = Bernoulli::new(0.5).unwrap();
    let not_sent = Bernoulli::new(0.3).unwrap();

    if corrupted.sample(&mut thread_rng()) {
        let length = buf[0] & 0x3f;
        buf[length as usize + 1] = 1;

        println!("UDT - Packet corrupted");
    }

    if not_sent.sample(&mut thread_rng()) {
        println!("UDT - Packet not sent");
        return;
    }

    socket.send_to(buf, addr).unwrap();
}

