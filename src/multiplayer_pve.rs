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
    self, get_randomized_port, EnemyMonsterSpawned, GameClient, PlayerType, ReadyToSpawnEnemy,
    ReadyToSpawnFriend,
};
use crate::monster::{
    get_monster_sprite_for_type, Boss, Defense, Element, Enemy, Health, Level, MonsterStats, Moves,
    PartyMonster, SelectedMonster, Strength,
};
use crate::multiplayer_pvp::convert_num_to_element;
use crate::multiplayer_waiting::{is_client, is_host};
use crate::networking::{
    BattleAction, BattleData, ClientActionEvent, HostActionEvent, InputActive, Message,
    MonsterTypeEvent, MultBattleBackground, MultBattleUIElement, MultEnemyHealth, MultEnemyMonster,
    MultFriendHealth, MultFriendMonster, MultMonster, MultPlayerHealth, MultPlayerMonster,
    PvETurnResultEvent, SelectedEnemyMonster, SelectedFriendMonster, TradingAvailable,
    MULT_BATTLE_BACKGROUND,
};
use crate::world::{GameProgress, PooledText, TextBuffer, TypeSystem, SPECIALS_PER_BATTLE};
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

// Host side:
// - Send health? to client in a BattleAction::StartTurn
//   + Flip TurnFlag to false
// - recv_packet receives BattleAction::FinishTurn which will contain the action the client took (0-3) and their stats
//   + Client stats will be stored via updating local entities
//   + Client's action will be sent to handler for FinishTurnEvent within the event
//   + They will be denied by keypress handler if they try to press again when not their turn
//   + Receiver will fire a FinishTurnEvent
// - handler function for FinishTurnEvent will run
//   + Queries ActionCache and uses the client action taken out of the event, both of which were setup prior
//   + Calculates local updates with `calculate_turn` and the above actions
//   + Updates stats locally based on result
//   + Sends a TurnsResult packet containing the calculated turn
//   + Flips TurnFlag to true
//
// Will look largely the same as PvP
// Client side:
// - recv_packet receives BattleAction::StartTurn
//   + flip TurnFlag to true
// - Picks their own action (0-3) (with client version of key_press_handler)
//   + client_action_handler needs to query the resource ActionCache(usize) to
//     get the action out of the recv_packet system to do turn calculation
//   + They will be denied by keypress handler if not their turn)
//   + Sends a BattleAction::FinishTurn with their own action and stats
//   + Flips TurnFlag to false
// - recv_packet receives BattleAction::TurnResult
//   + Contains data necessary to apply turn update locally
//   + Can either directly make these modifications in recv_packet or
//     fire a TurnResultEvent and have a client_do_turn_result_handler that handles.

// turn(host): choose action, disable turn, send stats
// turn(client): update stats, choose action, disable turn, send
// turn(host): calculate result, next turn...

#[derive(Component, Debug, Default)]
pub(crate) struct CachedData(BattleData);

#[derive(Component, Debug, Default)]
pub(crate) struct CachedAction(usize);

impl Plugin for MultPvEPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputActive>()
            .add_enter_system_set(
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
                    .with_system(
                        spawn_boss_monster_client.run_if_resource_exists::<ReadyToSpawnEnemy>(),
                    )
                    .with_system(
                        spawn_mult_friend_monster.run_if_resource_exists::<ReadyToSpawnFriend>(),
                    )
                    .with_system(update_mult_battle_stats)
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
                    .with_system(chat)
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
    mut game_progress: ResMut<GameProgress>,
) {
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // Init their inventories
    game_progress.spec_moves_left[0] = SPECIALS_PER_BATTLE;
    game_progress.player_inventory[0] = 4;
    game_progress.player_inventory[1] = 4;
    game_progress.turns_left_of_buff[0] = 0;
    game_progress.turns_left_of_buff[1] = 0;

    commands.insert_resource(TradingAvailable(true));

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

    let bytes = bincode::serialize(&selected_type).expect("couldn't serialize turn result");

    let msg = Message {
        action: BattleAction::FriendMonsterType,
        payload: bytes,
    };
    game_client
        .socket
        .udp_socket
        .send(&bincode::serialize(&msg).unwrap());
}

/// Function to receive messages from our teammate and
/// either handle them directly or (ideally) fire an event
/// to trigger a system to handle it.
pub(crate) fn recv_packets(
    game_client: Res<GameClient>,
    mut commands: Commands,
    mut monster_type_event: EventWriter<MonsterTypeEvent>,
    mut host_action_event: EventWriter<HostActionEvent>,
    mut turn_result_event: EventWriter<PvETurnResultEvent>,
    mut turn: ResMut<TurnFlag>,
    mut battle_data: ResMut<CachedData>,
    mut text_buffer: ResMut<TextBuffer>,
    mut game_progress: ResMut<GameProgress>,
    // Refers only to monster trading being available
    mut trading_available: ResMut<TradingAvailable>,
    mut selected_monster_query: Query<
        Entity,
        (
            With<SelectedMonster>,
            Without<SelectedFriendMonster>,
            Without<SelectedEnemyMonster>,
        ),
    >,
    mut friend_monster_query: Query<
        Entity,
        (
            With<SelectedFriendMonster>,
            Without<SelectedMonster>,
            Without<SelectedEnemyMonster>,
        ),
    >,
) {
    loop {
        let mut buf = [0; 512];
        match game_client.socket.udp_socket.recv(&mut buf) {
            Ok(msg) => {
                // info!("from here: {}, {:#?}", msg, &buf[..msg]);
                let decoded_msg: Message = bincode::deserialize(&buf[..msg]).unwrap();
                let action_type = decoded_msg.action.clone();
                info!("Action type: {:#?}", action_type);

                // Fill in event fires to handle incoming data
                if action_type == BattleAction::FriendMonsterType {
                    let monster_type = bincode::deserialize::<Element>(&decoded_msg.payload)
                        .expect("could not deserialize friend monster type");
                    // Create structs for opponent's monster
                    let friend_monster_stats = MonsterStats {
                        typing: monster_type,
                        lvl: Level { level: 1 },
                        hp: Health {
                            max_health: 100,
                            health: 100,
                        },
                        stg: Strength {
                            atk: 10,
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
                        .spawn_bundle(friend_monster_stats)
                        .insert(SelectedFriendMonster);

                    commands.insert_resource(ReadyToSpawnFriend {});
                } else if action_type == BattleAction::Initialize {
                    // Tell the host that we're ready to start, all significant information
                    // has been received client side.
                    commands.insert_resource(EnemyMonsterSpawned {});
                } else if action_type == BattleAction::BossMonsterType {
                    // only called for client
                    let monster_type = bincode::deserialize::<Element>(&decoded_msg.payload)
                        .expect("could not deserialize boss type");
                    let boss_monster_stats = MonsterStats {
                        typing: monster_type,
                        // payload just contains element at the moment
                        lvl: Level { level: 2 },
                        hp: Health {
                            max_health: 200,
                            health: 200,
                        },
                        stg: Strength {
                            atk: 10,
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
                        .spawn_bundle(boss_monster_stats)
                        .insert(SelectedEnemyMonster);

                    commands.insert_resource(ReadyToSpawnEnemy {});
                } else if action_type == BattleAction::StartTurn {
                    turn.0 = true;
                    trading_available.0 = false;
                    let text = PooledText {
                        text: "Your turn!".to_string(),
                        pooled: false,
                    };
                    text_buffer.bottom_text.push_back(text);
                } else if action_type == BattleAction::FinishTurn {
                    let client_action = decoded_msg.payload[0];
                    host_action_event.send(HostActionEvent(BattleData {
                        act: client_action,
                        atk: 0,
                        crt: 0,
                        def: 0,
                        ele: 0,
                    }));
                    trading_available.0 = false;
                    let text = PooledText {
                        text: "Your turn!".to_string(),
                        pooled: false,
                    };
                    text_buffer.bottom_text.push_back(text);
                } else if action_type == BattleAction::PvETurnResult {
                    // Will likely fire event with enough information for client
                    // to update their local stats for themselves, their friend, and
                    // the boss.
                    let payload = decoded_msg.payload;
                    let results_tuple = bincode::deserialize::<(isize, isize, isize)>(&payload)
                        .expect("could not deserialize turn result");
                    turn_result_event.send(PvETurnResultEvent(results_tuple));
                } else if action_type == BattleAction::Quit {
                    // Handle quit
                    info!("Other player disconnected...");
                    commands.insert_resource(NextState(GameState::Start));
                } else if action_type == BattleAction::TradeHeal {
                    // They gave us a heal!! So nice :)
                    game_progress.player_inventory[0] += 1;
                    // That consumed their turn :(
                    turn.0 = true;
                    trading_available.0 = false;
                    let text = PooledText {
                        text: "Received a heal item".to_string(),
                        pooled: false,
                    };
                    text_buffer.bottom_text.push_back(text);
                } else if action_type == BattleAction::TradeBuff {
                    // They gave us a buff!!! SO STRONG!!
                    game_progress.player_inventory[1] += 1;
                    turn.0 = true;
                    // only for monster trading
                    trading_available.0 = false;
                    let text = PooledText {
                        text: "Received a buff item".to_string(),
                        pooled: false,
                    };
                    text_buffer.bottom_text.push_back(text);
                } else if action_type == BattleAction::TradeMonster {
                    if friend_monster_query.is_empty() {
                        error!("cannot find friend monster");
                        return;
                    }
                    if selected_monster_query.is_empty() {
                        error!("cannot find our monster");
                        return;
                    }

                    let my_old_monster = selected_monster_query.single();
                    let friend_old_monster = friend_monster_query.single();

                    // commands.entity(my_old_monster).despawn_recursive();
                    // commands.entity(friend_old_monster).despawn_recursive();

                    // (Element of the receiver's new monster, Element of the sender's new monster)
                    let all_monster_types =
                        bincode::deserialize::<(Element, Element)>(&decoded_msg.payload)
                            .expect("could not deserialize monster trade");
                    let mynew_monster_stats = MonsterStats {
                        typing: all_monster_types.0,
                        lvl: Level { level: 1 },
                        hp: Health {
                            max_health: 100,
                            health: 100,
                        },
                        stg: Strength {
                            atk: 10,
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
                        .entity(my_old_monster)
                        .insert_bundle(mynew_monster_stats)
                        .insert(SelectedMonster);

                    // spawn the friend's monster
                    let friendnew_monster_stats = MonsterStats {
                        typing: all_monster_types.1,
                        lvl: Level { level: 1 },
                        hp: Health {
                            max_health: 100,
                            health: 100,
                        },
                        stg: Strength {
                            atk: 10,
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
                        .entity(friend_old_monster)
                        .insert_bundle(friendnew_monster_stats)
                        .insert(SelectedFriendMonster);
                } else if action_type == BattleAction::ChatMessage {
                    let payload = decoded_msg.payload;
                    let chat_msg = String::from_utf8_lossy(&payload).into_owned();
                    info!("got text: {}", &chat_msg);
                    let text = PooledText {
                        text: chat_msg,
                        pooled: false,
                    };
                    text_buffer.bottom_text.push_back(text);
                } else if action_type == BattleAction::EasterEggMessage {
                    let payload = decoded_msg.payload;
                    let chat_msg = String::from_utf8_lossy(&payload).into_owned();
                    info!("got text: {}", &chat_msg);
                    let text = PooledText {
                        text: chat_msg,
                        pooled: false,
                    };
                    text_buffer.easter_egg_ascii.push_back(text);
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
                        font_size: 30.0,
                        color: Color::BLACK,
                    },
                ),
                // health of player's monster
                TextSection::from_style(TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 30.0,
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
                        font_size: 30.0,
                        color: Color::BLACK,
                    },
                ),
                // health of player's monster
                TextSection::from_style(TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 30.0,
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
                        font_size: 30.0,
                        color: Color::BLACK,
                    },
                ),
                // health of opponent's monster
                TextSection::from_style(TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 30.0,
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
        (
            With<SelectedMonster>,
            Without<SelectedFriendMonster>,
            Without<SelectedEnemyMonster>,
        ),
    >,
    mut friend_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (
            With<SelectedFriendMonster>,
            Without<SelectedMonster>,
            Without<SelectedEnemyMonster>,
        ),
    >,
    mut turn: ResMut<TurnFlag>,
    game_client: Res<GameClient>,
    mut text_buffer: ResMut<TextBuffer>,
    mut game_progress: ResMut<GameProgress>,
    mut trading_available: ResMut<TradingAvailable>,
    mut input_active: ResMut<InputActive>,
) {
    if turn.0 && !input_active.0 {
        // This is client's turn
        // info!("Client may act");
        if input.just_pressed(KeyCode::A) {
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::FinishTurn,
                // payload: [0 as u8].to_vec(),
                payload: Vec::from([0_u8; 1]),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::E) {
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::FinishTurn,
                // payload: [0 as u8].to_vec(),
                payload: Vec::from([2_u8; 1]),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::S) {
            if game_progress.spec_moves_left[0] == 0 {
                // Cannot make special move
                let text = PooledText {
                    text: "No specials left...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
                return;
            }
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            game_progress.spec_moves_left[0] -= 1;
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::FinishTurn,
                // payload: [0 as u8].to_vec(),
                payload: Vec::from([3_u8; 1]),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::Q) {
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::Quit,
                // payload: [0 as u8].to_vec(),
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
            commands.insert_resource(NextState(GameState::Start));
        } else if input.just_pressed(KeyCode::Key1) {
            if game_progress.player_inventory[0] == 0 {
                // Not allowed to heal
                let text = PooledText {
                    text: "No heal items...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
                return;
            }

            // Heal item usage
            game_progress.player_inventory[0] -= 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            let text = PooledText {
                text: format!(
                    "Healed. {} heals remaining",
                    game_progress.player_inventory[0]
                ),
                pooled: false,
            };
            text_buffer.bottom_text.push_back(text);

            let msg = Message {
                action: BattleAction::FinishTurn,
                payload: Vec::from([4_u8; 1]),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::Key2) {
            if game_progress.player_inventory[1] == 0 {
                // Not allowed to buff
                let text = PooledText {
                    text: "No buff items...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
                return;
            }

            // Heal item usage
            game_progress.player_inventory[1] -= 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            let text = PooledText {
                text: format!(
                    "Buffed. {} buffs remaining",
                    game_progress.player_inventory[1]
                ),
                pooled: false,
            };
            text_buffer.bottom_text.push_back(text);

            let msg = Message {
                action: BattleAction::FinishTurn,
                payload: Vec::from([5_u8; 1]),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::Key3) {
            // Trade heal item
            if game_progress.player_inventory[0] == 0 {
                // Not allowed to heal
                let text = PooledText {
                    text: "No heal items to send...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
            }
            // Buff item usage
            game_progress.player_inventory[0] -= 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            let text = PooledText {
                text: "Sent heal item".to_string(),
                pooled: false,
            };
            text_buffer.bottom_text.push_back(text);
            // send data to host
            let msg = Message {
                action: BattleAction::TradeHeal,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::Key4) {
            // Trade buff item
            if game_progress.player_inventory[1] == 0 {
                // Not allowed to buff
                let text = PooledText {
                    text: "No buff items to send...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
            }
            // Buff item usage
            game_progress.player_inventory[1] -= 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            let text = PooledText {
                text: "Sent buff item".to_string(),
                pooled: false,
            };
            text_buffer.bottom_text.push_back(text);
            // send data to host
            let msg = Message {
                action: BattleAction::TradeBuff,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        }
    } // end turn check

    // Handle sending monster trade
    if input.just_pressed(KeyCode::M) && trading_available.0 && !input_active.0 {
        // trading monster
        if client_monster_query.is_empty() || friend_monster_query.is_empty() {
            return;
        }
        // get my monster type
        let client_old_element = *client_monster_query.single().4;
        let friend_old_element = *friend_monster_query.single().4;

        // destory my monster
        let client_old_entity = client_monster_query.single().3;
        let friend_old_entity = friend_monster_query.single().3;
        // commands.entity(host_old_entity).despawn_recursive();
        // commands.entity(friend_old_entity).despawn_recursive();

        let new_type: Element = rand::random();

        // generate a new monster
        commands
            .entity(client_old_entity)
            .remove_bundle::<MonsterStats>()
            .insert_bundle(MonsterStats {
                typing: new_type,
                lvl: Level { level: 1 },
                hp: Health {
                    max_health: 100,
                    health: 100,
                },
                stg: Strength {
                    atk: 10,
                    crt: 25,
                    crt_dmg: 2,
                },
                def: Defense {
                    def: 1,
                    crt_res: 10,
                },
                moves: Moves { known: 2 },
            })
            .insert(SelectedMonster);

        // make friend's new monster our old one
        commands
            .entity(friend_old_entity)
            .remove_bundle::<MonsterStats>()
            .insert_bundle(MonsterStats {
                typing: client_old_element,
                lvl: Level { level: 1 },
                hp: Health {
                    max_health: 100,
                    health: 100,
                },
                stg: Strength {
                    atk: 10,
                    crt: 25,
                    crt_dmg: 2,
                },
                def: Defense {
                    def: 1,
                    crt_res: 10,
                },
                moves: Moves { known: 2 },
            })
            .insert(SelectedFriendMonster);

        // send to friend
        let msg = Message {
            action: BattleAction::TradeMonster,
            payload: bincode::serialize(&(client_old_element, new_type))
                .expect("Cannot serialize monster type to trade"),
        };
        game_client
            .socket
            .udp_socket
            .send(&bincode::serialize(&msg).unwrap());
    }
}

/// System to update local stats with data given by host after a turn cycle finishes
pub(crate) fn client_end_turn_handler(
    mut commands: Commands,

    mut results_event: EventReader<PvETurnResultEvent>,
    mut client_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (
            With<SelectedMonster>,
            Without<SelectedEnemyMonster>,
            Without<SelectedFriendMonster>,
        ),
    >,
    mut enemy_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (
            Without<SelectedMonster>,
            With<SelectedEnemyMonster>,
            Without<SelectedFriendMonster>,
        ),
    >,
    mut friend_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (
            With<SelectedFriendMonster>,
            Without<SelectedMonster>,
            Without<SelectedEnemyMonster>,
        ),
    >,
    mut text_buffer: ResMut<TextBuffer>,
) {
    // get result data from event
    let mut wrapped_data: Option<(isize, isize, isize)> = None;
    for event in results_event.iter() {
        info!("Got action event");
        wrapped_data = Some(event.0);
        info!("Host received data: {:?}", wrapped_data.unwrap());
    }

    if wrapped_data.is_none() {
        return;
    }

    let data = wrapped_data.unwrap();

    let (mut client_hp, client_stg, client_def, _client_entity, client_element) =
        client_monster_query.single_mut();
    let (mut friend_hp, friend_stg, friend_def, _friend_entity, friend_element) =
        friend_monster_query.single_mut();
    let (mut enemy_hp, enemy_stg, enemy_def, _enemy_entity, enemy_element) =
        enemy_monster_query.single_mut();

    // update on our end
    client_hp.health -= data.1;
    friend_hp.health -= data.0;
    enemy_hp.health -= data.2;

    if enemy_hp.health <= 0 {
        let text = PooledText {
            text: "You won!".to_string(),
            pooled: false,
        };
        text_buffer.bottom_text.push_back(text);
        commands.insert_resource(NextState(GameState::Start));
    } else if client_hp.health <= 0 || friend_hp.health <= 0 {
        let text = PooledText {
            text: "You lost!".to_string(),
            pooled: false,
        };
        text_buffer.bottom_text.push_back(text);
        commands.insert_resource(NextState(GameState::Start));
    }
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
        (
            With<SelectedMonster>,
            Without<SelectedEnemyMonster>,
            Without<SelectedFriendMonster>,
        ),
    >,
    mut friend_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (
            With<SelectedFriendMonster>,
            Without<SelectedMonster>,
            Without<SelectedEnemyMonster>,
        ),
    >,
    mut turn: ResMut<TurnFlag>,
    game_client: Res<GameClient>,
    mut game_progress: ResMut<GameProgress>,
    mut battle_data: ResMut<CachedData>,
    mut host_cached_action: ResMut<CachedAction>,
    mut text_buffer: ResMut<TextBuffer>,
    mut trading_available: ResMut<TradingAvailable>,
    mut input_active: ResMut<InputActive>,
) {
    if turn.0 && !input_active.0 {
        // This is host's turn
        // info!("Host may act");
        if input.just_pressed(KeyCode::A) {
            host_cached_action.0 = 0;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::StartTurn,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::D) {
            host_cached_action.0 = 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::StartTurn,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::E) {
            // Elemental
            host_cached_action.0 = 2;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::StartTurn,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::S) {
            if game_progress.spec_moves_left[0] == 0 {
                // Cannot make special move
                let text = PooledText {
                    text: "No specials left...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
                return;
            }
            // Special move
            host_cached_action.0 = 3;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            game_progress.spec_moves_left[0] -= 1;
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::StartTurn,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::Key1) {
            if game_progress.player_inventory[0] == 0 {
                // Not allowed to heal
                let text = PooledText {
                    text: "No heal items...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
                return;
            }
            // Heal item usage
            host_cached_action.0 = 4;
            game_progress.player_inventory[0] -= 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            let text = PooledText {
                text: format!(
                    "Healed. {} heals remaining",
                    game_progress.player_inventory[0]
                ),
                pooled: false,
            };
            text_buffer.bottom_text.push_back(text);
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::StartTurn,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::Key2) {
            if game_progress.player_inventory[1] == 0 {
                // Not allowed to buff
                let text = PooledText {
                    text: "No buff items...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
            }
            // Buff item usage
            host_cached_action.0 = 5;
            game_progress.player_inventory[1] -= 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            let text = PooledText {
                text: format!(
                    "Buffed. {} buffs remaining",
                    game_progress.player_inventory[1]
                ),
                pooled: false,
            };
            text_buffer.bottom_text.push_back(text);
            // send startTurn to client
            // no data is needed, just inform the client
            let msg = Message {
                action: BattleAction::StartTurn,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::Q) {
            // Quit
            let msg = Message {
                action: BattleAction::Quit,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());

            commands.insert_resource(NextState(GameState::Start));
        } else if input.just_pressed(KeyCode::Key3) {
            // Trade heal item
            if game_progress.player_inventory[0] == 0 {
                // Not allowed to heal
                let text = PooledText {
                    text: "No heal items to send...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
            }
            // Buff item usage
            host_cached_action.0 = 6;
            game_progress.player_inventory[0] -= 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            let text = PooledText {
                text: "Sent heal item".to_string(),
                pooled: false,
            };
            text_buffer.bottom_text.push_back(text);
            // send data to client to client
            let msg = Message {
                action: BattleAction::TradeHeal,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        } else if input.just_pressed(KeyCode::Key4) {
            // Trade buff item
            if game_progress.player_inventory[1] == 0 {
                // Not allowed to heal
                let text = PooledText {
                    text: "No buff items to send...".to_string(),
                    pooled: false,
                };
                text_buffer.bottom_text.push_back(text);
            }
            // Buff item usage
            host_cached_action.0 = 7;
            game_progress.player_inventory[1] -= 1;
            // flip the turn flag
            turn.0 = false;
            trading_available.0 = false;
            let text = PooledText {
                text: "Sent buff item".to_string(),
                pooled: false,
            };
            text_buffer.bottom_text.push_back(text);
            // send data to client to client
            let msg = Message {
                action: BattleAction::TradeBuff,
                payload: Vec::new(),
            };
            game_client
                .socket
                .udp_socket
                .send(&bincode::serialize(&msg).unwrap());
        }
    }

    // Handle monster trading
    if input.just_pressed(KeyCode::M) && trading_available.0 && !input_active.0 {
        // trading monster
        if host_monster_query.is_empty() || friend_monster_query.is_empty() {
            return;
        }
        // get my monster type
        let host_old_element = *host_monster_query.single().4;
        let friend_old_element = *friend_monster_query.single().4;

        // destory my monster
        let host_old_entity = host_monster_query.single().3;
        let friend_old_entity = friend_monster_query.single().3;
        // commands.entity(host_old_entity).despawn_recursive();
        // commands.entity(friend_old_entity).despawn_recursive();

        let new_type: Element = rand::random();

        // generate a new monster
        commands
            .entity(host_old_entity)
            .remove_bundle::<MonsterStats>()
            .insert_bundle(MonsterStats {
                typing: new_type,
                lvl: Level { level: 1 },
                hp: Health {
                    max_health: 100,
                    health: 100,
                },
                stg: Strength {
                    atk: 10,
                    crt: 25,
                    crt_dmg: 2,
                },
                def: Defense {
                    def: 1,
                    crt_res: 10,
                },
                moves: Moves { known: 2 },
            })
            .insert(SelectedMonster);

        // make friend's new monster our old one
        commands
            .entity(friend_old_entity)
            .remove_bundle::<MonsterStats>()
            .insert_bundle(MonsterStats {
                typing: host_old_element,
                lvl: Level { level: 1 },
                hp: Health {
                    max_health: 100,
                    health: 100,
                },
                stg: Strength {
                    atk: 10,
                    crt: 25,
                    crt_dmg: 2,
                },
                def: Defense {
                    def: 1,
                    crt_res: 10,
                },
                moves: Moves { known: 2 },
            })
            .insert(SelectedFriendMonster);

        // send to friend
        let msg = Message {
            action: BattleAction::TradeMonster,
            payload: bincode::serialize(&(host_old_element, new_type))
                .expect("Cannot serialize monster type to trade"),
        };
        game_client
            .socket
            .udp_socket
            .send(&bincode::serialize(&msg).unwrap());
    }
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
        (
            With<SelectedMonster>,
            Without<SelectedEnemyMonster>,
            Without<SelectedFriendMonster>,
        ),
    >,
    mut enemy_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (
            Without<SelectedMonster>,
            Without<SelectedFriendMonster>,
            With<SelectedEnemyMonster>,
        ),
    >,
    // Should we have a friend monster query?
    // yes
    mut friend_monster_query: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (
            With<SelectedFriendMonster>,
            Without<SelectedMonster>,
            Without<SelectedEnemyMonster>,
        ),
    >,
    game_client: Res<GameClient>,
    type_system: Res<TypeSystem>,
    cached_host_action: Res<CachedAction>,
    mut text_buffer: ResMut<TextBuffer>,
    mut game_progress: ResMut<GameProgress>,
    mut turn: ResMut<TurnFlag>,
) {
    if friend_monster_query.is_empty()
        || enemy_monster_query.is_empty()
        || host_monster_query.is_empty()
    {
        warn!("One of the monsters missing from query");
        return;
    }

    // get client data from event
    // a little bit more than we need
    let mut wrapped_data: Option<BattleData> = None;
    for event in action_event.iter() {
        info!("Got action event");
        wrapped_data = Some(event.0);
        info!("Host received data: {:?}", wrapped_data.unwrap());
    }

    if wrapped_data.is_none() {
        return;
    }

    let data = wrapped_data.unwrap();

    let (mut host_hp, host_stg, host_def, _host_entity, host_element) =
        host_monster_query.single_mut();
    let (mut friend_hp, friend_stg, friend_def, _friend_entity, friend_element) =
        friend_monster_query.single_mut();
    let (mut enemy_hp, enemy_stg, enemy_def, _enemy_entity, enemy_element) =
        enemy_monster_query.single_mut();

    // Client buff
    if data.act == 5 {
        game_progress.turns_left_of_buff[1] = 3;
    }

    // Check whether host has buff damage
    let host_atk_modifier = if game_progress.turns_left_of_buff[0] > 0 {
        info!("Host buffed!");
        game_progress.turns_left_of_buff[0] -= 1;
        10
    } else {
        0
    };

    // Check whether client has buff damage
    let client_atk_modifier = if game_progress.turns_left_of_buff[1] > 0 {
        info!("Client buffed!");
        game_progress.turns_left_of_buff[1] -= 1;
        10
    } else {
        0
    };

    // boss choose action
    let mut enemy_action = rand::thread_rng().gen_range(0..=3);

    // Enemy cannot special if it is out of special moves
    if enemy_action == 3 && game_progress.spec_moves_left[1] == 0 {
        enemy_action = rand::thread_rng().gen_range(0..=2);
    }

    let enemy_act_string = if enemy_action == 0 {
        "Enemy attacks!".to_string()
    } else if enemy_action == 1 {
        "Enemy defends!".to_string()
    } else if enemy_action == 2 {
        "Enemy elemental!".to_string()
    } else {
        game_progress.spec_moves_left[1] -= 1;
        "Enemy special!".to_string()
    };

    let text = PooledText {
        text: format!("You attack! {}", enemy_act_string),
        pooled: false,
    };
    text_buffer.bottom_text.push_back(text);

    let enemy_message_bytes = enemy_act_string.into_bytes();
    let msg = Message {
        action: BattleAction::ChatMessage,
        payload: enemy_message_bytes,
    };
    game_client
        .socket
        .udp_socket
        .send(&bincode::serialize(&msg).unwrap());

    // calculate host's result to the boss
    let host_boss_result = pve_calculate_turn(
        (host_stg.atk + host_atk_modifier) as u8,
        host_stg.crt as u8,
        host_def.def as u8,
        *host_element as u8,
        cached_host_action.0 as u8,
        enemy_stg.atk as u8,
        enemy_stg.crt as u8,
        enemy_def.def as u8,
        *enemy_element as u8,
        enemy_action as u8,
        *type_system,
    );
    // calculate client's result to the boss
    let client_boss_result = pve_calculate_turn(
        (friend_stg.atk + client_atk_modifier) as u8,
        friend_stg.crt as u8,
        friend_def.def as u8,
        *friend_element as u8,
        data.act as u8,
        enemy_stg.atk as u8,
        enemy_stg.crt as u8,
        enemy_def.def as u8,
        *enemy_element as u8,
        enemy_action as u8,
        *type_system,
    );

    // let boss choose target, lets give them a little AI
    // dmg dealt to: (host, client, boss)
    let mut turn_result = (0, 0, host_boss_result.0 + client_boss_result.0);
    if client_boss_result.1 > host_boss_result.1 {
        turn_result.1 = client_boss_result.1;
    } else if client_boss_result.1 < host_boss_result.1 {
        turn_result.0 = host_boss_result.1;
    } else {
        // do an AOE?
        turn_result.0 = host_boss_result.1;
        turn_result.1 = client_boss_result.1;
    }

    // Check for heals
    if cached_host_action.0 == 4 {
        // We healed on this turn
        let heal_amount = 20 - turn_result.0;
        turn_result.0 = -heal_amount;
    }

    // Client heal
    if data.act == 4 {
        let heal_amount = 20 - turn_result.1;
        turn_result.1 = -heal_amount;
    }

    // update on our end
    host_hp.health -= turn_result.0;
    friend_hp.health -= turn_result.1;
    enemy_hp.health -= turn_result.2;

    if enemy_hp.health <= 0 {
        let text = PooledText {
            text: "You won!".to_string(),
            pooled: false,
        };
        text_buffer.bottom_text.push_back(text);
        commands.insert_resource(NextState(GameState::Start));
    } else if host_hp.health <= 0 || friend_hp.health <= 0 {
        let text = PooledText {
            text: "You lost!".to_string(),
            pooled: false,
        };
        text_buffer.bottom_text.push_back(text);
        commands.insert_resource(NextState(GameState::Start));
    }

    // send result to client
    let bytes = bincode::serialize(&turn_result).expect("couldn't serialize turn result");
    let msg = Message {
        action: BattleAction::PvETurnResult,
        payload: bytes,
    };
    game_client
        .socket
        .udp_socket
        .send(&bincode::serialize(&msg).unwrap());

    // flip the turn flag
    turn.0 = true;
}

/// Initialize a boss monster
pub(crate) fn create_boss_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<MultCamera>)>,
    game_client: Res<GameClient>,
    mut created_before: Query<(&Element, Entity), (With<MultEnemyMonster>)>,
    mut enemy_monster_query: Query<
        (Entity, &Element),
        (Without<SelectedMonster>, With<SelectedEnemyMonster>),
    >,
) {
    if cameras.is_empty() {
        error!("No spawned camera...?");
        return;
    }

    if (!created_before.is_empty()) {
        return;
    }

    let (ct, _) = cameras.single();

    let (enemy_entity, _enemy_element) = enemy_monster_query.single();

    commands
        .entity(enemy_entity)
        .insert_bundle(SpriteBundle {
            sprite: Sprite { ..default() },
            texture: asset_server.load(&get_monster_sprite_for_type(*_enemy_element)),
            transform: Transform::from_xyz(ct.translation.x + 400., ct.translation.y - 100., 5.),
            ..default()
        })
        .insert(MultEnemyMonster)
        .insert(MultMonster);

    // use this:
    // // Serialize an object
    // let bytes = bincode::serialize(&object).expect("couldn't serialize object");
    // // Deserialize a payload
    // let thing: T = bincode::deserialize::<T where T: Serialize>(&payload).expect("could not deserialize object T");
    // also in discord

    let bytes = bincode::serialize(&_enemy_element).expect("couldn't serialize object");

    let msg = Message {
        action: BattleAction::BossMonsterType,
        payload: bytes,
    };
    game_client
        .socket
        .udp_socket
        .send(&bincode::serialize(&msg).unwrap());
}

/// System to spawn the enemy monster on screen for the client, who confirms that they recieved that info.
pub(crate) fn spawn_boss_monster_client(
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
        error!("No selected monster in boss spawner...?");
        return;
    }

    let (ct, _) = cameras.single();

    let (selected_type, selected_monster) = selected_monster_query.single();

    commands
        .entity(selected_monster)
        .insert_bundle(SpriteBundle {
            sprite: Sprite { ..default() },
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
        error!("No selected monster in player spawner...?");
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
        error!("No selected monster in friend spawner...?");
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

    // commands.remove_resource::<ReadyToSpawnFriend>();
}

/// System to update GUI statistics
pub(crate) fn update_mult_battle_stats(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
    mut set: ParamSet<(
        Query<
            &mut Health,
            (
                With<SelectedMonster>,
                Without<SelectedEnemyMonster>,
                Without<SelectedFriendMonster>,
            ),
        >,
        Query<
            &mut Health,
            (
                With<SelectedEnemyMonster>,
                Without<SelectedMonster>,
                Without<SelectedFriendMonster>,
            ),
        >,
        Query<
            &mut Health,
            (
                With<SelectedFriendMonster>,
                Without<SelectedMonster>,
                Without<SelectedEnemyMonster>,
            ),
        >,
    )>,
    mut player_health_text_query: Query<
        &mut Text,
        (
            With<MultPlayerHealth>,
            Without<MultEnemyHealth>,
            Without<MultFriendHealth>,
        ),
    >,
    mut friend_health_text_query: Query<
        &mut Text,
        (
            With<MultFriendHealth>,
            Without<MultEnemyHealth>,
            Without<MultPlayerHealth>,
        ),
    >,
    mut enemy_health_text_query: Query<
        &mut Text,
        (
            With<MultEnemyHealth>,
            Without<MultPlayerHealth>,
            Without<MultFriendHealth>,
        ),
    >,
) {
    let mut my_health = 0;
    let mut enemy_health = 0;
    let mut friend_health = 0;
    for my_monster in set.p0().iter_mut() {
        my_health = my_monster.health;
    }

    for mut text in &mut player_health_text_query {
        text.sections[1].value = format!("{}", my_health);
    }

    for enemy_monster in set.p1().iter_mut() {
        enemy_health = enemy_monster.health;
    }

    for mut text in &mut enemy_health_text_query {
        text.sections[1].value = format!("{}", enemy_health);
    }

    for friend_monster in set.p2().iter_mut() {
        friend_health = friend_monster.health;
    }

    for mut text in &mut friend_health_text_query {
        text.sections[1].value = format!("{}", friend_health);
    }
}

/// Despawn all data associated with the battle and reset
/// anything in game_progress in case player goes to play another singleplayer
/// or different kind of multiplayer game.
fn despawn_mult_battle(
    mut commands: Commands,
    background_query: Query<Entity, With<MultBattleBackground>>,
    monster_query: Query<Entity, With<MultMonster>>,
    mult_battle_ui_element_query: Query<Entity, With<MultBattleUIElement>>,
    selected_monster_query: Query<Entity, (With<SelectedMonster>)>,
    mut game_progress: ResMut<GameProgress>,
) {
    if background_query.is_empty() {
        error!("background is not here!");
    }

    background_query.for_each(|background| {
        commands.entity(background).despawn();
    });

    monster_query.for_each(|monster| {
        commands.entity(monster).despawn_recursive();
    });

    if mult_battle_ui_element_query.is_empty() {
        error!("ui elements are not here!");
    }

    mult_battle_ui_element_query.for_each(|mult_battle_ui_element| {
        commands.entity(mult_battle_ui_element).despawn_recursive();
    });

    selected_monster_query.for_each(|monster| commands.entity(monster).despawn_recursive());

    game_progress.spec_moves_left[0] = SPECIALS_PER_BATTLE;
    game_progress.player_inventory[0] = 0;
    game_progress.player_inventory[1] = 0;
    game_progress.turns_left_of_buff[0] = 0;
    game_progress.turns_left_of_buff[1] = 0;

    commands.remove_resource::<ReadyToSpawnEnemy>();
    commands.remove_resource::<ReadyToSpawnFriend>();
}

pub(crate) fn chat(
    mut char_event: EventReader<ReceivedCharacter>,
    input: Res<Input<KeyCode>>,
    mut string: Local<String>,
    mut input_active: ResMut<InputActive>,
    game_client: Res<GameClient>,
) {
    if input_active.0 {
        for ev in char_event.iter() {
            // info!("Got char: '{}'", ev.char);
            string.push(ev.char);
        }

        if input.just_pressed(KeyCode::Return) {
            // info!("Text input: {}", *string);
            input_active.0 = false;
            let mut chat_msg = string.to_owned();
            info!("Text input: {}", &chat_msg);
            chat_msg.pop();
            if chat_msg.eq("dan") {
                let msg = Message {
                    action: BattleAction::EasterEggMessage,
                    payload: format!(
                        "
                      .-'~~~-.
                    .'o  oOOOo`.
                   :~~~-.oOo   o`.
                    `. ~ ~-.  oOOo.
                      `.; ~ ~.  OO:
                      .'  ;-- `.o.'
                     ,'  ; ~~--'~
                     ;  ;
                     "
                    )
                    .into_bytes(),
                };
                game_client
                    .socket
                    .udp_socket
                    .send(&bincode::serialize(&msg).unwrap());
                string.clear();
            } else if chat_msg.eq("gavin") {
                let msg = Message {
                    action: BattleAction::EasterEggMessage,
                    payload: format!(
                        "   
                          .+------+ 
                        .' |    .'| 
                       +---+--+'  | 
                       |   |  |   | 
                       |  ,+--+---+ 
                       |.'    | .' 
                       +------+'   
                       "
                    )
                    .into_bytes(),
                };
                game_client
                    .socket
                    .udp_socket
                    .send(&bincode::serialize(&msg).unwrap());
                string.clear();
            } else if chat_msg.eq("chris") {
                let msg = Message {
                    action: BattleAction::EasterEggMessage,
                    payload: format!(
                        // "
                        // _______________________________________
                        // | The Industrial Revolution and its     |
                        // | consequences have been a disaster for |
                        // | the human race                        |
                        //  ---------------------------------------
                        "
                            |   ^__^
                            |__ (oo)|_______
                                (__)|       )---
                                    ||----w |
                                    ||     ||
                        "
                    )
                    .into_bytes(),
                };
                game_client
                    .socket
                    .udp_socket
                    .send(&bincode::serialize(&msg).unwrap());
                string.clear();
            } else {
                let msg = Message {
                    action: BattleAction::ChatMessage,
                    payload: chat_msg.into_bytes(),
                };
                game_client
                    .socket
                    .udp_socket
                    .send(&bincode::serialize(&msg).unwrap());
                string.clear();
            }
        }
    }

    if input.just_pressed(KeyCode::C) {
        input_active.0 = true;
    }
}

/// # Placeholder
///
/// ⚠ This function is currently just a placeholder.
///
/// Function to calculate the results of a full turn cycle, including the combined damage
/// by the host AND client, as well as the boss (enemy) and which player the boss chose to attack.
///
/// ## Return
/// (isize, isize): damage dealt to boss monster, damage dealt to friendly monster
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
) -> (isize, isize) {
    if player_action == 1 || enemy_action == 1 {
        // if either side defends this turn will not have any damage on either side
        return (0, 0);
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

    (result.0 as isize, result.1 as isize)
}
