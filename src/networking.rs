use bevy::prelude::{Component, Entity};
use serde::{Deserialize, Serialize};

/// Bevy Event wrapper around BattleActions
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct BattleEvent(pub BattleAction);

/// BattleActions as an enum separates the desired result for data sent to apply
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum BattleAction {
    MonsterStats,
    MonsterType,
    FriendMonsterType,
    BossMonsterType,
    Attack,
    Defend,
    Special,
    Quit,
    StartTurn,
    FinishTurn,
    TurnResult,
    PvETurnResult,
    Initialize,
    Heal
}
// Message structs represent the data within the message on a larger sense of scale.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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
        Self { action, payload }
    }
}

// Shared networking components and data

pub(crate) const MULT_BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";

/// Represents the type of mode the host and/or client chose to play
#[derive(Clone, Copy, Debug)]
pub(crate) enum MultiplayerMode {
    PvP,
    PvE,
}

/// Resource to contain the mode selected by a player
///
/// Default initialization will select PvP
#[derive(Clone, Copy, Debug)]
pub(crate) struct MultiplayerModeSelected {
    pub(crate) mode: MultiplayerMode,
}

impl Default for MultiplayerModeSelected {
    fn default() -> Self {
        Self {
            mode: MultiplayerMode::PvP,
        }
    }
}

#[derive(Component)]
pub(crate) struct MultBattleBackground;

#[derive(Component)]
pub(crate) struct MultMonster;

#[derive(Component)]
pub(crate) struct MultPlayerMonster;

#[derive(Component)]
pub(crate) struct MultFriendMonster;

#[derive(Component)]
pub(crate) struct MultEnemyMonster;

#[derive(Component)]
pub(crate) struct SelectedEnemyMonster;

#[derive(Component)]
pub(crate) struct SelectedFriendMonster;

// Unit structs to help identify the specific UI components for player's or enemy's monster health/level
// since there may be many Text components
#[derive(Component)]
pub(crate) struct MultPlayerHealth;

#[derive(Component)]
pub(crate) struct MultFriendHealth;

#[derive(Component)]
pub(crate) struct MultEnemyHealth;

#[derive(Component)]
pub(crate) struct MultBattleUIElement;

#[derive(Debug)]
pub(crate) struct MonsterTypeEvent {
    pub(crate) message: Message,
}
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BattleData {
    pub(crate) act: u8,
    pub(crate) atk: u8,
    pub(crate) crt: u8,
    pub(crate) def: u8,
    pub(crate) ele: u8,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct ClientActionEvent(pub(crate) BattleData);

#[derive(Debug, Clone, Copy)]
pub(crate) struct HostActionEvent(pub(crate) BattleData);

#[derive(Debug, Clone, Copy)]
pub(crate) struct TurnResultEvent(pub(crate) (isize,isize));

#[derive(Debug, Clone, Copy)]
pub(crate) struct PvETurnResultEvent(pub(crate) (isize,isize,isize));
