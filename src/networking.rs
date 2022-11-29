// Networking helper structs and functions
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::{io};
use crate::game_client::*;

// use std::io::Bytes;
// MultiplayerActions as an enum separates the desired result for data sent to apply
pub enum MultiplayerActions {
    Initialize,
    MonsterStats,
    MonsterType,
    Attack,
    Defend,
    Heal,
    Special,
}

// Message structs represent the data within the message on a larger sense of scale.
pub struct Message {
    /// The destination to send the message.
    pub destination: SocketAddr,
    // The action ID to identify what data was sent
    pub action: MultiplayerActions,
    // The data sent itself.
    //pub payload: Bytes,
}

// function for initializing a new Message
impl Message {
    /// Creates and returns a new Message.
    pub(crate) fn new(destination: SocketAddr, action: MultiplayerActions, payload: &[u8]) -> Self {
        Self {
            destination,
            action,
            // payload: use std::io::Bytes;,
            
        }
    }
}

pub enum MultiplayerEvent {
    // A message was received from a client
    // Message(SocketAddr, MultiplayerActions, Bytes),
    // A new client has connected to us
    Connected(SocketAddr),
    // A client has disconnected from us
    Disconnected(SocketAddr),
    // An error occurred while receiving a message
    RecvError(io::Error),
    // An error occurred while sending a message
    // optional, setup message
    SendError(io::Error),
}
