use std::net::{SocketAddr};
use std::{io};
use serde::{Serialize, Deserialize};

// MultiplayerActions as an enum separates the desired result for data sent to apply
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum BattleEvent {
    Initialize,
    MonsterStats,
    MonsterType,
    Attack,
    Defend,
    Heal,
    Special,
}

// pub enum NetworkEvent {
//     Message(SocketAddr, Bytes),
//     RecvError(io::Error),
//     SendError(io::Error, Message)
// }

// Message structs represent the data within the message on a larger sense of scale.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    /// The destination to send the message.
    // The action ID to identify what data was sent
    pub event: BattleEvent,
    // The data sent itself.
    pub payload: Vec<u8>,
}

// function for initializing a new Message
impl Message {
    /// Creates and returns a new Message.
    pub(crate) fn new(event: BattleEvent, payload: Vec<u8>) -> Self {
        Self {
            // destination,
            event,
            payload,
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
