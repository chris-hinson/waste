use crate::{monster::*, world::NUM_ITEM_TYPES};
use bevy::prelude::*;
use rand::Rng;

pub(crate) const NPC_PATH: &str = "characters/npc_sprite.png";

/// Component to mark quest-giving NPCs and store data associated
/// with them, such as their X,Y location and the quest they are giving.
#[derive(Component, Debug)]
pub(crate) struct NPC {
    // pub(crate) transform: Transform,
    pub(crate) quest: Quest,
}

/// Struct to store data for quests given by NPCs to the player
#[derive(Clone, Copy, Debug)]
pub(crate) struct Quest {
    pub(crate) target: Element,
    pub(crate) reward: usize,
    pub(crate) reward_amount: usize,
}

impl Quest {
    /// Create a new quest object with a random target monster and random reward
    pub(crate) fn random() -> Self {
        let target = rand::random();
        let reward = rand::thread_rng().gen_range(0..NUM_ITEM_TYPES) as usize;
        let reward_amount = rand::thread_rng().gen_range(1..=5) as usize;
        Self {
            target,
            reward,
            reward_amount,
        }
    }
}
