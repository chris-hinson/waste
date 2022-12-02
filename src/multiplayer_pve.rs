#![allow(unused)]
use crate::backgrounds::{Tile, WIN_H, WIN_W};
use crate::camera::MultCamera;
use crate::game_client::{
    self, get_randomized_port, GameClient, Package, PlayerType, ReadyToSpawnEnemy, ReadyToSpawnFriend
};
use crate::monster::{
    get_monster_sprite_for_type, Boss, Defense, Element, Enemy, Health, Level, MonsterStats, Moves,
    PartyMonster, SelectedMonster, Strength,
};
use crate::networking::{
    BattleAction, Message, MultBattleBackground, MultBattleUIElement, MultEnemyHealth,
    MultEnemyMonster, MultMonster, MultPlayerHealth, MultPlayerMonster, MultFriendHealth, 
    MultFriendMonster, SelectedEnemyMonster, SelectedFriendMonster, MULT_BATTLE_BACKGROUND, 
};
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
            SystemSet::new()
                .with_system(setup_mult_battle) // .with_system(send_monster)
                .with_system(setup_pve_battle_stats),
        )
        .add_system_set(
            ConditionSet::new()
                // Only run handlers on MultiplayerBattle state
                .run_in_state(GameState::MultiplayerPvEBattle)
                .with_system(spawn_mult_player_monster)
                .with_system(spawn_mult_friend_monster.run_if_resource_exists::<ReadyToSpawnFriend>())
                //.with_system(spawn_mult_enemy_monster.run_if_resource_exists::<ReadyToSpawnEnemy>())
                .with_system(mult_key_press_handler)
                .with_system(recv_packets)
                .into(),
        )
        .add_exit_system(GameState::MultiplayerPvEBattle, despawn_mult_battle);
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
        action: BattleAction::FriendMonsterType,
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
                if action_type == BattleAction::FriendMonsterType {
                    // Create structs for opponent's monster
                    let friend_monster_stats = MonsterStats {
                        typing: convert_num_to_element(payload),
                        lvl: Level { level: 1 },
                        hp: Health {
                            max_health: 10,
                            health: 10,
                        },
                        stg: Strength {
                            atk: 2,
                            crt: 25,
                            crt_dmg: 2,
                        },
                        def: Defense {
                            def: 1,
                            crt_res: 10,
                        },
                        moves: Moves { known: 2 },
                    };
                    commands
                        .spawn()
                        .insert_bundle(friend_monster_stats)
                        .insert(SelectedFriendMonster);

                    commands.insert_resource(ReadyToSpawnFriend {});
                }
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

pub(crate) fn setup_pve_battle_stats(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_client: Res<GameClient>,
) {
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // health header for player's monster
                TextSection::new(
                    "Your Health:",
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
            }),
        )
        .insert(MultPlayerHealth)
        .insert(MultBattleUIElement);
    
        commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // health header for player's monster
                TextSection::new(
                    "Friend Health:",
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
                    left: Val::Px(425.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(MultFriendHealth)
        .insert(MultBattleUIElement);

    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // health header for opponent's monster
                TextSection::new(
                    "Boss Health:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // health of opponent's monster
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
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        //.insert(MonsterBundle::default())
        .insert(MultEnemyHealth)
        .insert(MultBattleUIElement);
}

fn convert_num_to_element(num: usize) -> Element {
    match num {
        0 => Element::Scav,
        1 => Element::Growth,
        2 => Element::Ember,
        3 => Element::Flood,
        4 => Element::Rad,
        5 => Element::Robot,
        6 => Element::Clean,
        7 => Element::Filth,
        _ => std::process::exit(256),
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
        
        BattleAction::FriendMonsterType => {
            let payload = message.payload;
        }
        BattleAction::BossMonsterType => {
            let payload = message.payload;
        }
        BattleAction::Defend => todo!(),
        BattleAction::Heal => todo!(),
        BattleAction::Special => todo!(),
        // this is a prank don't todo!
        BattleAction::MonsterType => todo!(),
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

pub(crate) fn spawn_mult_player_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<MultCamera>)>,
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

pub(crate) fn spawn_mult_friend_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<MultCamera>)>,
    selected_monster_query: Query<(&Element, Entity), (With<SelectedFriendMonster>)>,
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

    
    let (selected_type, selected_monster) = selected_monster_query.single();

    commands
        .entity(selected_monster)
        .remove_bundle::<SpriteBundle>()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                flip_y: false, // flips our little buddy, you guessed it, in the y direction
                flip_x: true, // guess what this does
                ..default()
            },
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x - 50., ct.translation.y - 100., 5.),
            ..default()
        })
        .insert(MultFriendMonster)
        .insert(MultMonster);

    commands.remove_resource::<ReadyToSpawnFriend>();
}

fn despawn_mult_battle(
    mut commands: Commands,
    // camera_query: Query<Entity,  With<MenuCamera>>,
    // background_query: Query<Entity, With<MultMenuBackground>>,
    // mult_ui_element_query: Query<Entity, With<MultMenuUIElement>>
) {
}
