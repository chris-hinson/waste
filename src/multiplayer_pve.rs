// These will be the only warnings we want to suppress at the end of 
// the day. During development, it is fine to allow(unused), as long as 
// warnings are fixed before pull to main.
// #![allow(unused_must_use)]
// #![allow(unused_mut)]
// #![allow(unused_parens)]
// Development warning suppression
#![allow(unused)]
use crate::camera::MultCamera;
use crate::game_client::{
    self, get_randomized_port, GameClient, EnemyMonsterSpawned, PlayerType, ReadyToSpawnEnemy,
    ReadyToSpawnFriend,
};
use crate::monster::{
    get_monster_sprite_for_type, Boss, Defense, Element, Enemy, Health, Level, MonsterStats, Moves,
    PartyMonster, SelectedMonster, Strength,
};
use crate::multiplayer_pvp::convert_num_to_element;
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
                    // Will likely fire event with enough information for client
                    // to update their local stats for themselves, their friend, and
                    // the boss.
                }
            }
            // Error handler
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

pub(crate) fn init_host_turnflag(mut commands: Commands) {
    commands.insert_resource(TurnFlag(true));
}

pub(crate) fn init_client_turnflag(mut commands: Commands) {
    commands.insert_resource(TurnFlag(false));
}

/// System to handle keypresses (actions) by the client
/// 
/// Needs to give the host enough information to do full turn cycle calculation
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

/// System to update local stats with data given by host after a turn cycle finishes
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

/// System to handle keypresses (actions) by client
/// 
/// This will most likely have to send data back to the client to let them know it is their turn
/// as well as update some local stats.
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

/// System to finish a turn cycle's handling
/// 
/// This might be complex. It needs to potentially wrap all of the actions
/// taken by both the host and the client up into a finalized result of damage dealt
/// (or healing experienced) on all three sides, host/client/boss. It should be the 
/// **ONLY FUNCTION IN A TURN CYCLE** which calls `mult_calculate_turn`, otherwise we
/// run into critical desync issues. All calculations and RNG is done host side and then the
/// final results are given back to the client so they can update their game's stats using 
/// `client_end_turn_handler`.
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

/// Initialize a boss monster
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

/// # Deprecated
/// Delete ASAP
/// 
/// System to spawn the enemy monster on screen. Should be handled instead by `create_boss_monster`.
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


/// System to spawn our monster, on both client and host side.
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
                flip_y: false,
                flip_x: true, 
                ..default()
            },
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x - 400., ct.translation.y - 100., 5.),
            ..default()
        })
        .insert(MultPlayerMonster)
        .insert(MultMonster);
}

/// System to spawn our friend's monster, on both client and host side.
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

/// Despawn all data associated with the battle and reset
/// anything in game_progress in case player goes to play another singleplayer
/// or different kind of multiplayer game.
fn despawn_mult_battle(
    mut commands: Commands,
    // camera_query: Query<Entity,  With<MenuCamera>>,
    // background_query: Query<Entity, With<MultMenuBackground>>,
    // mult_ui_element_query: Query<Entity, With<MultMenuUIElement>>
) {
}

/// # Placeholder
/// 
/// ⚠ This function is currently just a placeholder. 
/// 
/// Function to calculate the results of a full turn cycle, including the combined damage
/// by the host AND client, as well as the boss (enemy) and which player the boss chose to attack.
fn pve_calculate_turn(
    player_atk: u8,
    player_crt: u8,
    player_def: u8,
    player_type: u8,
    player_action: u8,
    enemy_atk: u8,
    enemy_crt: u8,
    enemy_def: u8,
    enemy_type: u8,
    enemy_action: u8,
    type_system: TypeSystem,
) -> (isize, isize, usize) {
    if player_action == 1 || enemy_action == 1 {
        // if either side defends this turn will not have any damage on either side
        return (0, 0, 0);
    }
    // More actions can be added later, we can also consider decoupling the actions from the damage
    let mut result = (
        0, // Your damage to enemy
        0, // Enemy's damage to you
    );
    // player attacks
    // If our attack is less than the enemy's defense, we do 0 damage
    if player_atk <= enemy_def {
        result.0 = 0;
    } else {
        // if we have damage, we do that much damage
        // I've only implemented crits for now, dodge and element can follow
        result.0 = (player_atk - enemy_def) as usize;
        if player_crt > 15 {
            // calculate crit chance and apply crit damage
            let crit_chance = player_crt - 15;
            let crit = rand::thread_rng().gen_range(0..=100);
            if crit <= crit_chance {
                info!("You had a critical strike!");
                result.0 *= 2;
            }
        }
    }
    // same for enemy
    if enemy_atk <= player_def {
        result.1 = 0;
    } else {
        result.1 = (enemy_atk - player_def) as usize;
        if enemy_crt > 15 {
            let crit_chance = enemy_crt - 15;
            let crit = rand::thread_rng().gen_range(0..=100);
            if crit <= crit_chance {
                info!("Enemy had a critical strike!");
                result.1 *= 2;
            }
        }
    }

    if player_action == 2 {
        // Elemental move
        result.0 = (type_system.type_modifier[player_type as usize][enemy_type as usize]
            * result.0 as f32)
            .trunc() as usize;
    } else if player_action == 3 {
        // Multi-move
        // Do an attack first
        result.0 += pve_calculate_turn(
            player_atk,
            player_crt,
            player_def,
            player_type,
            0,
            enemy_atk,
            enemy_crt,
            enemy_def,
            enemy_type,
            enemy_action,
            type_system,
        )
        .0 as usize;
        // Then simulate elemental
        result.0 = (type_system.type_modifier[player_type as usize][enemy_type as usize]
            * result.0 as f32)
            .trunc() as usize;
    }

    if enemy_action == 2 {
        result.1 = (type_system.type_modifier[enemy_type as usize][player_type as usize]
            * result.1 as f32)
            .trunc() as usize;
    } else if enemy_action == 3 {
        // Multi-move
        // Do an attack first
        result.1 += pve_calculate_turn(
            player_atk,
            player_crt,
            player_def,
            player_type,
            player_action,
            enemy_atk,
            enemy_crt,
            enemy_def,
            enemy_type,
            0,
            type_system,
        )
        .1 as usize;
        // Then simulate elemental
        result.1 = (type_system.type_modifier[enemy_type as usize][player_type as usize]
            * result.1 as f32)
            .trunc() as usize;
    }

    // Handle heals, buffs, or trades, which don't actually
    // do any damage, since this function still needs called if 
    // one player/enemy decides not to deal damage (as the others might have).
    if player_action == 4 || player_action == 5 {
        result.0 = 0_usize;
    }

    if enemy_action == 4 || enemy_action == 5 {
        result.1 = 0_usize;
    }

    (result.0 as isize, result.1 as isize, rand::thread_rng().gen_range(0..=1_usize))
}
