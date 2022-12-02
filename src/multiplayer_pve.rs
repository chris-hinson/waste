#![allow(unused)]
use crate::backgrounds::{Tile, WIN_H, WIN_W};
use crate::camera::MultCamera;
use crate::game_client::{
    self, get_randomized_port, GameClient, PlayerType, ReadyToSpawnEnemy,
};
use crate::monster::{
    get_monster_sprite_for_type, Boss, Defense, Element, Enemy, Health, Level, MonsterStats, Moves,
    PartyMonster, SelectedMonster, Strength,
};
use crate::networking::{BattleAction, Message, MultBattleBackground, MULT_BATTLE_BACKGROUND};
use crate::GameState;
use bevy::{prelude::*, ui::*};
use bincode;
use iyes_loopless::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, UdpSocket};
use std::str::from_utf8;
use std::{io, thread};

pub struct MultPvEPlugin;

impl Plugin for MultPvEPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system_set(
            GameState::MultiplayerPvEBattle,
            SystemSet::new().with_system(setup_mult_battle), // .with_system(send_monster)
        )
        .add_system_set(
            ConditionSet::new()
                // Only run handlers on MultiplayerBattle state
                .run_in_state(GameState::MultiplayerPvEBattle)
                .with_system(mult_key_press_handler)
                .with_system(recv_packets)
                .into(),
        )
        .add_exit_system(GameState::MultiplayerPvPBattle, despawn_mult_battle);
    }
}

pub(crate) fn setup_mult_battle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, (With<Camera2d>, Without<MultCamera>)>,
    game_client: Res<GameClient>,
    selected_monster_query: Query<(&Element), (With<SelectedMonster>)>,
) {
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    //creates camera for multiplayer battle background
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MultCamera);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(MULT_BATTLE_BACKGROUND),
            transform: Transform::from_xyz(0., 0., 2.),
            ..default()
        })
        .insert(MultBattleBackground);

    // send type of monster to other player
    let (selected_type) = selected_monster_query.single();
    let num_type = *selected_type as usize;

    let msg = Message {
        action: BattleAction::MonsterType,
        payload: num_type.to_ne_bytes().to_vec(),
    };
    game_client
        .socket
        .udp_socket
        .send(&bincode::serialize(&msg).unwrap());
}

/// Function to receive messages from our teammate and
/// either handle them directly or (ideally) fire an event
/// to trigger a system to handle it.
pub(crate) fn recv_packets(game_client: Res<GameClient>, mut commands: Commands) {
    loop {
        let mut buf = [0; 512];
        match game_client.socket.udp_socket.recv(&mut buf) {
            Ok(msg) => {
                info!("from here: {}, {:#?}", msg, &buf[..msg]);
                let decoded_msg: Message = bincode::deserialize(&buf[..msg]).unwrap();
                let action_type = decoded_msg.action;
                let payload = usize::from_ne_bytes(decoded_msg.payload.try_into().unwrap());
                info!("{:#?}", action_type);
                info!("{:#?}", payload);

                // Fill in event fires to handle incoming data
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::WouldBlock {
                    // An ACTUAL error occurred
                    error!("{}", err);
                }
                // Done reading, break;
                break;
            }
        }
    }
}

/// Function to send message to our teammate
pub(crate) fn send_message(message: Message) {
    match message.action {
        BattleAction::Attack => {
            let payload = message.payload;
            //info!("{:#?}", from_utf8(&payload).unwrap());
        }
        BattleAction::Initialize => todo!(),
        BattleAction::MonsterStats => todo!(),
        BattleAction::MonsterType => {
            let payload = message.payload;
        }
        BattleAction::Defend => todo!(),
        BattleAction::Heal => todo!(),
        BattleAction::Special => todo!(),
    }
}

pub(crate) fn mult_key_press_handler(
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut my_monster: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (With<SelectedMonster>),
    >,
    asset_server: Res<AssetServer>,
    game_client: Res<GameClient>,
) {
    if input.just_pressed(KeyCode::A) {
        // ATTACK
        info!("Attack!");

        send_message(Message {
            // destination: (game_client.socket.socket_addr),
            action: (BattleAction::Attack),
            payload: "i attacked the enemy".to_string().into_bytes(),
        });
    } else if input.just_pressed(KeyCode::Q) {
        // ABORT
        info!("Quit!")
    } else if input.just_pressed(KeyCode::D) {
        // DEFEND
        info!("Defend!")
    } else if input.just_pressed(KeyCode::E) {
        // ELEMENTAL
        info!("Elemental attack!")
    }
}

fn despawn_mult_battle(
    mut commands: Commands,
    // camera_query: Query<Entity,  With<MenuCamera>>,
    // background_query: Query<Entity, With<MultMenuBackground>>,
    // mult_ui_element_query: Query<Entity, With<MultMenuUIElement>>
) {
}
