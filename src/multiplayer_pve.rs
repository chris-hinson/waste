#![allow(unused)]
use crate::backgrounds::{Tile, WIN_H, WIN_W};
use crate::camera::MultCamera;
use crate::game_client::{
    self, get_randomized_port, GameClient, EnemyMonsterSpawned, PlayerType, ReadyToSpawnEnemy,
    ReadyToSpawnFriend,
};
use crate::monster::{
    get_monster_sprite_for_type, Boss, Defense, Element, Enemy, Health, Level, MonsterStats, Moves,
    PartyMonster, SelectedMonster, Strength,
};
use crate::multiplayer_waiting::{is_client, is_host};
use crate::networking::{
    BattleAction, BattleData, Message, MultBattleBackground, MultBattleUIElement, MultEnemyHealth,
    MultEnemyMonster, MultFriendHealth, MultFriendMonster, MultMonster, MultPlayerHealth,
    MultPlayerMonster, SelectedEnemyMonster, SelectedFriendMonster, MULT_BATTLE_BACKGROUND,
    ClientActionEvent, HostActionEvent, MonsterTypeEvent, PvETurnResultEvent
};
use crate::world::{PooledText, TextBuffer, TypeSystem, GameProgress, SPECIALS_PER_BATTLE};
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

/// Flag to determine whether this
#[derive(Clone, Copy)]
pub(crate) struct TurnFlag(pub(crate) bool);

#[derive(Component, Debug, Default)]
pub(crate) struct CachedData(BattleData);

#[derive(Component, Debug, Default)]
pub(crate) struct CachedAction(usize);

impl Plugin for MultPvEPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system_set(
            GameState::MultiplayerPvEBattle,
            SystemSet::new()
                .with_system(setup_mult_battle) // .with_system(send_monster)
                .with_system(setup_pve_battle_stats)
                .with_system(init_host_turnflag.run_if(is_host))
                .with_system(init_client_turnflag.run_if(is_client)),
        )
        .add_system_set(
            ConditionSet::new()
                // Only run handlers on MultiplayerBattle state
                .run_in_state(GameState::MultiplayerPvEBattle)
                .with_system(spawn_mult_player_monster)
                .with_system(create_boss_monster.run_if(is_host))
                .with_system(spawn_mult_enemy_monster.run_if_resource_exists::<ReadyToSpawnEnemy>())
                .with_system(spawn_mult_friend_monster.run_if_resource_exists::<ReadyToSpawnFriend>())
                .with_system(mult_key_press_handler)
                .with_system(
                    host_action_handler
                        .run_if_resource_exists::<EnemyMonsterSpawned>()
                        .run_if_resource_exists::<TurnFlag>()
                        .run_if(is_host),
                )
                .with_system(
                    host_end_turn_handler
                        .run_if_resource_exists::<TurnFlag>()
                        .run_if(is_host),
                )
                .with_system(
                    client_action_handler
                        .run_if_resource_exists::<TurnFlag>()
                        .run_if(is_client),
                )
                .with_system(
                    client_end_turn_handler
                        .run_if_resource_exists::<TurnFlag>()
                        .run_if(is_client),
                )
                .with_system(recv_packets.run_if_resource_exists::<TurnFlag>())
                .into(),
        )
        .init_resource::<CachedData>()
        .init_resource::<CachedAction>()
        .add_event::<MonsterTypeEvent>()
        .add_event::<HostActionEvent>()
        .add_event::<ClientActionEvent>()
        .add_event::<PvETurnResultEvent>()
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
pub(crate) fn recv_packets(game_client: Res<GameClient>, mut commands: Commands, 
    mut monster_type_event: EventWriter<MonsterTypeEvent>,
    mut host_action_event: EventWriter<HostActionEvent>,
    mut turn_result_event: EventWriter<PvETurnResultEvent>,
    mut turn: ResMut<TurnFlag>,
    mut battle_data: ResMut<CachedData>,
    mut text_buffer: ResMut<TextBuffer>,) {
    loop {
        let mut buf = [0; 512];
        match game_client.socket.udp_socket.recv(&mut buf) {
            Ok(msg) => {
                info!("from here: {}, {:#?}", msg, &buf[..msg]);
                let decoded_msg: Message = bincode::deserialize(&buf[..msg]).unwrap();
                let action_type = decoded_msg.action.clone();
                let payload = usize::from_ne_bytes(decoded_msg.payload.try_into().unwrap());
                info!("Action type: {:#?}", action_type);
                info!("Payload is: {:#?}", payload);

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
                
                } else if action_type == BattleAction::Initialize {
                    commands.insert_resource(EnemyMonsterSpawned {});                    
                } else if action_type == BattleAction::BossMonsterType {
                    // only called for client
                    let boss_monster_stats = MonsterStats {
                        typing: convert_num_to_element(payload),
                        // payload just contains element at the moment
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
                        .insert_bundle(boss_monster_stats)
                        .insert(SelectedEnemyMonster);

                    commands.insert_resource(ReadyToSpawnEnemy {});

                } else if action_type == BattleAction::PvETurnResult {

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

/// (outdated) Function to send message to our teammate 
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
        BattleAction::Quit => todo!(),
        BattleAction::StartTurn => todo!(),
        BattleAction::FinishTurn => todo!(),
        BattleAction::PvETurnResult => todo!(),
        BattleAction::TurnResult => todo!(),
    }
}

pub(crate) fn init_host_turnflag(mut commands: Commands) {
    commands.insert_resource(TurnFlag(true));
}

pub(crate) fn init_client_turnflag(mut commands: Commands) {
    commands.insert_resource(TurnFlag(false));
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
        // info!("Attack!");

        // send_message(Message {
        //     // destination: (game_client.socket.socket_addr),
        //     action: (BattleAction::Attack),
        //     payload: "i attacked the enemy".to_string().into_bytes(),
        // });
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

fn client_action_handler(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut client_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        With<SelectedMonster>,
    >,
    mut turn: ResMut<TurnFlag>,
    game_client: Res<GameClient>,
    mut text_buffer: ResMut<TextBuffer>,
    mut game_progress: ResMut<GameProgress>,
) {

}

pub(crate) fn client_end_turn_handler(
    mut commands: Commands,
    // mut action_event: EventReader<ClientActionEvent>,
    mut results_event: EventReader<PvETurnResultEvent>,
    mut client_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        With<SelectedMonster>,
    >,
    mut enemy_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (Without<SelectedMonster>, With<SelectedEnemyMonster>),
    >,
    mut text_buffer: ResMut<TextBuffer>,
) {

}

fn host_action_handler(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut host_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (With<SelectedMonster>),
    >,
    mut turn: ResMut<TurnFlag>,
    game_client: Res<GameClient>,
    mut game_progress: ResMut<GameProgress>,
    mut battle_data: ResMut<CachedData>,
    mut host_cached_action: ResMut<CachedAction>,
    mut text_buffer: ResMut<TextBuffer>,
) {

}

pub(crate) fn host_end_turn_handler(
    mut commands: Commands,
    mut action_event: EventReader<HostActionEvent>,
    mut host_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (With<SelectedMonster>),
    >,
    mut enemy_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (Without<SelectedMonster>, With<SelectedEnemyMonster>),
    >,
    game_client: Res<GameClient>,
    type_system: Res<TypeSystem>,
    cached_host_action: Res<CachedAction>,
    mut text_buffer: ResMut<TextBuffer>,
    mut game_progress: ResMut<GameProgress>,
) {

}

pub(crate) fn create_boss_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<MultCamera>)>,
    game_client: Res<GameClient>,
    selected_monster_query: Query<(&Element, Entity), (With<SelectedEnemyMonster>)>,
    created_before: Query<(&Element, Entity), (With<MultEnemyMonster>)>,
) {
    if cameras.is_empty() {
        error!("No spawned camera...?");
        return;
    }

    if selected_monster_query.is_empty() {
        error!("No selected monster...?");
        return;
    }

    if (!created_before.is_empty()) {
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
                flip_x: false,  // guess what this does
                ..default()
            },
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x + 400., ct.translation.y - 100., 5.),
            ..default()
        })
        .insert(MultEnemyMonster)
        .insert(MultMonster);

        let num_type = *selected_type as usize;
        let msg = Message {
            action: BattleAction::BossMonsterType,
            payload: num_type.to_ne_bytes().to_vec(),
        };
        game_client
            .socket
            .udp_socket
            .send(&bincode::serialize(&msg).unwrap());
}

pub(crate) fn spawn_mult_enemy_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<MultCamera>)>,
    game_client: Res<GameClient>,
    selected_monster_query: Query<(&Element, Entity), (With<SelectedEnemyMonster>)>,
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
                flip_x: false, // guess what this does
                ..default()
            },
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x + 400., ct.translation.y - 100., 5.),
            ..default()
        })
        .insert(MultEnemyMonster)
        .insert(MultMonster);

    commands.remove_resource::<ReadyToSpawnEnemy>();


    let num_type = *selected_type as usize;
        let msg = Message {
            action: BattleAction::Initialize,
            payload: num_type.to_ne_bytes().to_vec(),
        };
        game_client
            .socket
            .udp_socket
            .send(&bincode::serialize(&msg).unwrap());
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
                flip_x: true,  // guess what this does
                ..default()
            },
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x - 100., ct.translation.y - 100., 5.),
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
