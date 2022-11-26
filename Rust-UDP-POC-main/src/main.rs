mod player2;
mod player1;

use local_ip_address::local_ip;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use crate::player2::{player2, create_player2_connection};
use crate::player1::{player1, create_player1_connection};

#[allow(dead_code)]
#[allow(unused)]

const TOGGLE: &str = "h";
const SIZE: usize = 50;


fn main() {
    let (sx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (stx, rrx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let udp_conn_player1 = create_player1_connection();
    let udp_conn_player2 = create_player2_connection();

    thread::spawn(move || {
         player2(udp_conn_player2, sx, rrx).expect("Thread for player 2 could not be spawned");
    });

    let ip_addr_player2 = get_ip_addr_player2();
    udp_conn_player1.connect(ip_addr_player2).expect("Player 1 couldn't be connected to Player 2");

    udp_conn_player1.send(b"message from player 1 to player 2").expect("Couldn't send message from player 1 -> player 2");


    let mut buf = [0; 100];
    match udp_conn_player1.recv(&mut buf) {
        Ok(received) => println!("received {received} bytes {:?}", String::from_utf8((&buf[0..received]).to_vec()).unwrap()),
        Err(e) => println!("recv function failed: {e:?}"),
    }
}

pub(crate) fn get_ip_addr_player1() -> SocketAddr {
    let my_local_ip = local_ip().unwrap();
    let mut ip_addr = Ipv4Addr::new(127, 0, 0, 1);
    if let IpAddr::V4(ipv4) = my_local_ip {
        ip_addr = ipv4;
    }
    let socket = SocketAddr::new(IpAddr::from(ip_addr), 8080);
    socket
}

pub(crate) fn get_ip_addr_player2() -> SocketAddr {
    let my_local_ip = local_ip().unwrap();
    let mut ip_addr = Ipv4Addr::new(127, 0, 0, 1);
    if let IpAddr::V4(ipv4) = my_local_ip {
        ip_addr = ipv4;
    }
    let socket = SocketAddr::new(IpAddr::from(ip_addr), 9800);
    socket
}
