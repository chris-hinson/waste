use std::net::UdpSocket;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use crate::{get_ip_addr_player1, get_ip_addr_player2};

const SIZE: usize = 50;

pub(crate) fn create_player2_connection() -> UdpSocket {
    // client ip addr is same as host w/ diff port
    let ip_addr_player2 = get_ip_addr_player2();
    // create socket for client
    let udp_conn_client = UdpSocket::bind(ip_addr_player2).unwrap();
    // thread for client udp socket
    udp_conn_client
}

pub(crate) fn player2(socket: UdpSocket, sx: Sender<String>, rrx: Receiver<String>) -> std::io::Result<()> {
    // let mut buf = [0; 10];
    // let (number_of_bytes, src_addr) = udp_conn_client.recv_from(&mut buf)
    //     .expect("Didn't receive data");
    // let filled_buf = &mut buf[..number_of_bytes];
    // println!("{}, {}", number_of_bytes, src_addr);

    println!("Player 2 connection is active: ");

    socket.send_to(b"message from player 2 to player 1", get_ip_addr_player1()).expect("Couldn't send message from player 2 -> player 1");

    loop {
        let mut buf = [0 as u8; SIZE];
        let (num_bytes, src_addr) = socket.recv_from(&mut buf)?;
        let msg = String::from_utf8((&buf[0..num_bytes]).to_vec()).unwrap();
        println!("{}, {}", msg, src_addr);

        //send client data through channel
        match sx.send(String::from(msg)){
            Ok(_) => { }
            Err(e) => {
                println!("Error sending msg: {}", e)
            }
        }
        // match rx.try_recv() {
        //     Ok(msg) => {
        //         let e = String::from("Error: ".to_owned() + &msg);
        //         socket.send_to(e.as_bytes(), src_addr).expect("TODO: panic message");
        //     }
        //     Err(e) => {
        //         //no message in queue, do nothing
        //     }
        // }
    }
}