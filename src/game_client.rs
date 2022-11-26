use std::fmt;
use std::net::{UdpSocket, SocketAddr, Ipv4Addr, IpAddr};
use std::sync::mpsc::{Receiver, Sender};

use bevy::prelude::{Query, Component, info, FromWorld};
use local_ip_address::local_ip;
use rand::seq::SliceRandom;

const SIZE: usize = 50;

pub(crate) enum PlayerType {
    Host,
    Client,
}

#[derive(Debug)]
pub(crate) struct SocketInfo {
    pub(crate) addr: SocketAddr,
    pub(crate) udp_socket: UdpSocket
}

pub(crate) struct GameClient {
    pub(crate) socket: SocketInfo,
    pub(crate) player_type: PlayerType,
    pub(crate) udp_channel: UdpChannel
}

#[derive(Debug)]
pub(crate) struct Package {
    pub(crate) message: String,
    pub(crate) sender: Option<Sender<Package>>,
}

impl Package {
    pub(crate) fn new(message: String, sender: Option<Sender<Package>>) -> Self {
        Package {
            message,
            sender,
        }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub(crate) struct UdpChannel {
    pub(crate) sx: Sender<Package>,
    pub(crate) rx: Receiver<Package>,
}

unsafe impl Send for UdpChannel {}
unsafe impl Sync for UdpChannel {}


pub(crate) fn get_addr() -> SocketAddr {
    let port_list = vec![9800, 8081, 8082, 8083, 8084, 8085, 8086, 8087, 8088, 8089, 8090];
    // let port = 9800;
    let my_local_ip = local_ip().unwrap();
    let mut ip_addr = Ipv4Addr::new(127, 0, 0, 1);
    if let IpAddr::V4(ipv4) = my_local_ip {
        ip_addr = ipv4;
    }
    let socket = SocketAddr::new(IpAddr::from(ip_addr), *port_list.choose(&mut rand::thread_rng()).unwrap());
    // let socket = SocketAddr::new(IpAddr::from(ip_addr), port);

    socket
}