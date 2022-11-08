use std::net::{UdpSocket, SocketAddr};
use std::sync::mpsc::{Receiver, Sender};

use bevy::prelude::{Query, Component, info};

const SIZE: usize = 50;

pub(crate) enum PlayerType {
    Host,
	Client,
}

pub(crate) struct Socket {
	pub(crate) addr: SocketAddr,
	pub(crate) udp_socket: UdpSocket
}

#[derive(Component)]
pub(crate) struct GameClient {
	pub(crate) socket: Socket,
	pub(crate) player_type: PlayerType,
    pub(crate) udp_channel: UdpChannel
}

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

#[derive(Component)]
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

pub(crate) fn send_msg_to_client(mut game_client_query: Query<&mut GameClient>) {

    if game_client_query.is_empty() {
        info!("no matches for host!");
        return;
    }

    let mut game_client = game_client_query.get_single_mut().unwrap();

    game_client.socket.udp_socket.send(b"message from host to client").expect("Msg couldnt be sent from host to client");
}



pub (crate) fn receive_msg(
    mut game_client_query: Query<&mut GameClient>
) {
    
     if game_client_query.is_empty() {
        info!("no matches for host!");
        return;
    }

    let game_client = game_client_query.get_single_mut().unwrap();
    loop {
        let mut buf = [0 as u8; SIZE];
        let (num_bytes, src_addr) = game_client.socket.udp_socket.recv_from(&mut buf).expect("Msg could not be received for some reason");
        let msg = String::from_utf8((&buf[0..num_bytes]).to_vec()).unwrap();
        println!("{}, {}", msg, src_addr);
    }
}