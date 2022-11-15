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

pub(crate) struct UdpChannel {
    pub(crate) sx: Sender<Package>,
    pub(crate) rx: Receiver<Package>,
}

unsafe impl Send for UdpChannel {}
unsafe impl Sync for UdpChannel {}


// fn socket_controller(
//     socket: UdpSocket,
//     game_client_query: Query<&GameClient>,
// ) {
//     println!("{:?}", socket.local_addr());

    // if game_client_query.is_empty() {
    //     info!("no game client");
    //     return;
    // }

    // let game_client = game_client_query.get_single().unwrap();

    // for received in game_client.udp_channel.rx.try_recv() {
    //     println!("Got this in new thread: {}", received.message);
    //     println!("{:?}", received.sender);
    //     let main_gsx = received.sender.expect("main thread's sender not here");

    //     let response_back_to_main = Package::new(String::from("RESPONSE FROM THREAD HERE"), Some(game_client.udp_channel.sx.clone()));
    //     main_gsx.send(response_back_to_main).expect("panic message");
    // }








// }

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