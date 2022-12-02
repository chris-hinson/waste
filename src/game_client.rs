use rand::seq::SliceRandom;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

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
}

/// this is inserted as a resource when the player selects "Join Game"
/// since they want to be the client. this allows the client to move to the multiplayer battle stage
pub(crate) struct ClientMarker {}
pub(crate) struct HostMarker {}
pub(crate) struct HostNotReady {}
pub(crate) struct HostReady {}

pub(crate) struct ReadyToSpawnEnemy {}
pub(crate) struct ReadyToSpawnFriend {}


#[derive(Debug)]
pub(crate) struct Package {
    pub(crate) message: String,
}

impl Package {
    pub(crate) fn new(message: String) -> Self {
        Package { message }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Choose a random port of the known, normally open UDP ports
pub(crate) fn get_randomized_port() -> i32 {
    let port_list = vec![
        9800, 8081, 8082, 8083, 8084, 8085, 8086, 8087, 8088, 8089, 8090,
    ];
    *port_list.choose(&mut rand::thread_rng()).unwrap()
}

/// Get the local address to bind a socket to
pub(crate) fn get_addr() -> Vec<SocketAddr> {
    // let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    // let socket_addr = SocketAddr::new(ip_addr, 0 as u16);
    // socket_addr
    vec![
        SocketAddr::from(([127,0,0,1], 9800)),
        SocketAddr::from(([127,0,0,1], 8081)),
        SocketAddr::from(([127,0,0,1], 8082)),
        SocketAddr::from(([127,0,0,1], 8083)),
        SocketAddr::from(([127,0,0,1], 8084)),
        SocketAddr::from(([127,0,0,1], 8085)),
        SocketAddr::from(([127,0,0,1], 8086)),
        SocketAddr::from(([127,0,0,1], 8087)),
        SocketAddr::from(([127,0,0,1], 8088)),
        SocketAddr::from(([127,0,0,1], 8089)),
        SocketAddr::from(([127,0,0,1], 8090)),
    ]
}
