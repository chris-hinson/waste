use std::net::{UdpSocket, SocketAddr};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use bevy::prelude::{Query, Component, info, Plugin, App};

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
}



pub(crate) fn socket_controller(socket: UdpSocket, sx: Sender<String>, mrx: Receiver<String>) {
    //println!("{:?}", socket.local_addr());

    sx.send("test from new thread to main thread".parse().unwrap()).expect("couldnt send msg from new to main thread");

    for received in mrx {
        println!("Got this in new thread: {}", received);
    }




}

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