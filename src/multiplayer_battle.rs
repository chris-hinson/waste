#![allow(unused)]
use bevy::{prelude::*, ui::*};
use bytes::Bytes;
use iyes_loopless::prelude::*;
use rand::Rng;
use crate::game_client::{GameClient, self, PlayerType, Package, get_randomized_port};
use crate::monster::{
    get_monster_sprite_for_type, Boss, Defense, Element, Enemy, Health, Level, MonsterStats,
    PartyMonster, SelectedMonster, Strength,
};
use crate::networking::{BattleEvent, Message};
use crate::{
	GameState
};
use std::str::from_utf8;
use std::{io, thread};
use std::net::{UdpSocket, Ipv4Addr};
use std::sync::mpsc::{Receiver, Sender, self};
use crate::camera::{MultCamera};
use crate::backgrounds::{
	WIN_H, WIN_W, 
	Tile
};

const MULT_BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";

#[derive(Component)]
pub(crate) struct MultBattleBackground;

#[derive(Component)]
pub(crate) struct MultMonster;

#[derive(Component)]
pub(crate) struct MultPlayerMonster;

// Unit structs to help identify the specific UI components for player's or enemy's monster health/level
// since there may be many Text components
#[derive(Component)]
pub (crate) struct MultPlayerHealth;

#[derive(Component)]
pub (crate) struct MultPlayerLevel;

#[derive(Component)]
pub(crate) struct MultBattleUIElement;

#[derive(Component)]
pub(crate) struct MultPlayerType;

pub(crate) struct AttackEvent(Entity);

pub(crate) struct DefendEvent(Entity);

pub(crate) struct HealEvent(Entity);

pub struct MultBattlePlugin;

// Builds plugin for multiplayer battles
impl Plugin for MultBattlePlugin {
	fn build(&self, app: &mut App) {
		app
		.add_enter_system_set(GameState::MultiplayerBattle, 
            SystemSet::new()
                .with_system(setup_mult_battle)
                .with_system(setup_mult_battle_stats)
        )
		.add_system_set(ConditionSet::new()
			// Only run handlers on MultiplayerBattle state
			.run_in_state(GameState::MultiplayerBattle)
                .with_system(spawn_mult_player_monster)
                .with_system(update_mult_battle_stats)
                .with_system(mult_key_press_handler)
                .with_system(recv_packets)
        .into())
		.add_exit_system(GameState::MultiplayerBattle, despawn_mult_battle);
	}
}


pub(crate) fn recv_packets(game_client: Res<GameClient>) {
    loop {
        let mut buf = [0; 512];
        match game_client.socket.udp_socket.recv(&mut buf) {
            Ok(msg) => {
                println!("received {msg} bytes. The msg is: {}", from_utf8(&buf[..msg]).unwrap());
                info!("confirmation: entered recv packets");
                let data = from_utf8(&buf[..msg]).unwrap().to_string();
                info!(data);
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::WouldBlock { 
                    // An ACTUAL error occurred
                    error!("{}", err);
                }
                
                break;
            }
        }
    }
}

pub(crate) fn send_message(message: Message) {
    match message.event {
        BattleEvent::Attack => {
            let payload = message.payload;
            info!("{:#?}", from_utf8(&payload).unwrap());
        }
        BattleEvent::Initialize => todo!(),
        BattleEvent::MonsterStats => todo!(),
        BattleEvent::MonsterType => todo!(),
        BattleEvent::Defend => todo!(),
        BattleEvent::Heal => todo!(),
        BattleEvent::Special => todo!(),
    }
}


pub(crate) fn setup_mult_battle(mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, (With<Camera2d>, Without<MultCamera>)>,
    game_client: Res<GameClient>,
) { 

    cameras.for_each(|camera| {
		commands.entity(camera).despawn();
	});

    //creates camera for multiplayer battle background
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MultCamera);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load(MULT_BATTLE_BACKGROUND),
        transform: Transform::from_xyz(0., 0., 2.),
        ..default()
    })  
    .insert(MultBattleBackground);

}

pub(crate) fn setup_mult_battle_stats(
    mut commands: Commands, 
	asset_server: Res<AssetServer>,
    game_client: Res<GameClient>,
) {
    commands.spawn_bundle(
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            // health header for player's monster
            TextSection::new(
                "Health:",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::BLACK,
                },
            ),
            // health of player's monster
            TextSection::from_style(TextStyle {
                font: asset_server.load("buttons/joystix monospace.ttf"),
                font_size: 40.0,
                color: Color::BLACK,
            }),
        ])
        .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
        ),
    )
    .insert(MultPlayerHealth)
    .insert(MultBattleUIElement);
    
}

pub(crate) fn update_mult_battle_stats(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
    mut set: ParamSet<(
        Query<&mut Health, With<SelectedMonster>>,
    )>,
    mut player_health_text_query: Query<&mut Text, (With<MultPlayerHealth>)>
) {
    let mut my_health = 0;
    for my_monster in set.p0().iter_mut() {
        my_health = my_monster.health;
    }

    for mut text in &mut player_health_text_query {
        text.sections[1].value = format!("{}", my_health);
    }
}

pub(crate) fn spawn_mult_player_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<
        (&Transform, Entity),
        (With<MultCamera>),
    >,
    selected_monster_query: Query<(&Element, Entity), (With<SelectedMonster>)>,
) {
    if cameras.is_empty() {
        error!("No spawned camera...?");
        return;
    }

    if selected_monster_query.is_empty() {
        error!("No selected monster...?");
        return;
    }

    let (ct, _) = cameras.single();

    // why doesn't this update
    let (selected_type, selected_monster) = selected_monster_query.single();

    commands
        .entity(selected_monster)
        .remove_bundle::<SpriteBundle>()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                flip_y: false, // flips our little buddy, you guessed it, in the y direction
                flip_x: true,  // guess what this does
                ..default()
            },
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x - 400., ct.translation.y - 100., 5.),
            ..default()
        })
        .insert(MultPlayerMonster)
        .insert(MultMonster);
}


// TODO: spawn enemy's monster when data is sent from other player
pub(crate) fn spawn_mult_enemy_monster(mut commands: Commands) {

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
    if input.just_pressed(KeyCode::A) { // ATTACK
        info!("Attack!");

        send_message(Message { 
            destination: (game_client.socket.socket_addr), 
            event: (BattleEvent::Attack), 
            payload: "i attacked you".to_string().into_bytes()
        });
    }
    else if input.just_pressed(KeyCode::Q) { // ABORT
        info!("Quit!")
    } 
    else if input.just_pressed(KeyCode::D) { // DEFEND
        info!("Defend!")

    } 
    else if input.just_pressed(KeyCode::E) { // ELEMENTAL
        info!("Elemental attack!")
    }
}

fn despawn_mult_battle(mut commands: Commands,
	// camera_query: Query<Entity,  With<MenuCamera>>,
	// background_query: Query<Entity, With<MultMenuBackground>>,
    // mult_ui_element_query: Query<Entity, With<MultMenuUIElement>>
){

}
