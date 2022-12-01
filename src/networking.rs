use std::net::{SocketAddr};
use std::{io};
use serde::{Serialize, Deserialize};

/// Bevy Event wrapper around BattleActions
pub struct BattleEvent(BattleAction);

/// BattleActions as an enum separates the desired result for data sent to apply
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum BattleAction {
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
    pub action: BattleAction,
    // The data sent itself.
    pub payload: Vec<u8>,
}

// function for initializing a new Message
impl Message {
    /// Creates and returns a new Message.
    pub(crate) fn new(action: BattleAction, payload: Vec<u8>) -> Self {
        Self {
            // destination,
            action,
            payload,
        }
    }
}
