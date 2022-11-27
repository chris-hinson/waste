use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender};
use rand::seq::SliceRandom;

const SIZE: usize = 50;

#[derive(PartialEq)]
pub(crate) enum PlayerType {
    Host,
    Client,
}

#[derive(Debug)]
pub(crate) struct SocketInfo {
    pub(crate) socket_addr: SocketAddr,
    pub(crate) udp_socket: UdpSocket,
}

pub(crate) struct GameClient {
    pub(crate) socket: SocketInfo,
    pub(crate) player_type: PlayerType,
    pub(crate) udp_channel: UdpChannel,
}

/// this is inserted as a resource when the player selects "Join Game"
/// since they want to be the client. this allows the client to move to the multiplayer battle stage
pub(crate) struct ClientMarker {}

#[derive(Debug)]
pub(crate) struct Package {
    pub(crate) message: String,
    pub(crate) sender: Option<Sender<Package>>,
}

impl Package {
    pub(crate) fn new(message: String, sender: Option<Sender<Package>>) -> Self {
        Package { message, sender }
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


pub(crate) fn get_randomized_port() -> i32 {
    let port_list = vec![9800, 8081, 8082, 8083, 8084, 8085, 8086, 8087, 8088, 8089, 8090];
    *port_list.choose(&mut rand::thread_rng()).unwrap()
}

pub(crate) fn get_addr() -> SocketAddr {
    let port_list = vec![9800, 8081, 8082, 8083, 8084, 8085, 8086, 8087, 8088, 8089, 8090];
    let mut ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_addr = SocketAddr::new(ip_addr, *port_list.choose(&mut rand::thread_rng()).unwrap());
    socket_addr
}
