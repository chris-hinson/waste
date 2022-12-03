#![allow(unused)]
use crate::backgrounds::{Tile, WIN_H, WIN_W};
use crate::camera::MultCamera;
use crate::game_client::{
    self, get_randomized_port, GameClient, PlayerType, ReadyToSpawnEnemy, EnemyMonsterSpawned
};
use crate::monster::{
    get_monster_sprite_for_type, Boss, Defense, Element, Enemy, Health, Level, MonsterStats, Moves,
    PartyMonster, SelectedMonster, Strength,
};
use crate::networking::{
    BattleAction, Message, MultBattleBackground, MultBattleUIElement, MultEnemyHealth,
    MultEnemyMonster, MultMonster, MultPlayerHealth, MultPlayerMonster, SelectedEnemyMonster,
    MULT_BATTLE_BACKGROUND, MonsterTypeEvent, AttackEvent, ElementalAttackEvent, DefendEvent,
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

pub struct MultPvPPlugin;

// Builds plugin for multiplayer battles
impl Plugin for MultPvPPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system_set(
            GameState::MultiplayerPvPBattle,
            SystemSet::new()
                .with_system(setup_mult_battle)
                .with_system(setup_mult_battle_stats), // .with_system(send_monster)
        )
        .add_system_set(
            ConditionSet::new()
                
                // Only run handlers on MultiplayerBattle state
                .run_in_state(GameState::MultiplayerPvPBattle)
                    .with_system(spawn_mult_player_monster)
                    .with_system(spawn_mult_enemy_monster.run_if_resource_exists::<ReadyToSpawnEnemy>())
                    .with_system(update_mult_battle_stats)
                    .with_system(mult_key_press_handler.run_if_resource_exists::<EnemyMonsterSpawned>())
                    .with_system(recv_packets)
                .into(),
        )
            .add_system(handle_monster_type_event)
        .add_exit_system(GameState::MultiplayerPvPBattle, despawn_mult_battle);
    }
}

pub(crate) fn recv_packets(
    
    game_client: Res<GameClient>, 
    
    mut commands: Commands,
    mut monster_type_event: EventWriter<MonsterTypeEvent>,
    mut attack_event: EventWriter<AttackEvent>,
    mut elemental_attack_event: EventWriter<ElementalAttackEvent>,
    mut defend_event: EventWriter<DefendEvent>,
    mut player_monster: Query<
    (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
    (With<SelectedMonster>)>,
    mut enemy_monster: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (Without<SelectedMonster>, With<SelectedEnemyMonster>)>
    ) {
    loop {
        let mut buf = [0; 512];
        match game_client.socket.udp_socket.recv(&mut buf) {
            Ok(msg) => {
                //info!("from here: {}, {:#?}", msg, &buf[..msg]);
                let deserialized_msg: Message = bincode::deserialize(&buf[..msg]).unwrap();
                let action_type = deserialized_msg.action.clone();
                info!("Action type: {:#?}", action_type);

                // In an actual Bevy event system rather than handling each possible action
                // as it is received in this handler (to avoid having massively bloated handlers
                // like the ones in battle.rs 😢), what this system should do is fire a Bevy event
                // specific to each type of action and with the necessary information to handle it wrapped
                // inside. That way, some other Bevy system can handle this work ONLY IF the event was
                // fired to tell it to do so.
                // https://docs.rs/bevy/latest/bevy/ecs/event/struct.EventWriter.html
                // https://docs.rs/bevy/latest/bevy/ecs/event/struct.EventReader.html
                // https://bevy-cheatbook.github.io/programming/events.html
                if action_type == BattleAction::MonsterType {
                    let payload = usize::from_ne_bytes(deserialized_msg.payload.clone().try_into().unwrap());
                    monster_type_event.send(MonsterTypeEvent {message: deserialized_msg.clone() });
                }
                else if action_type == BattleAction::Attack {
                    // let payload = isize::from_ne_bytes(deserialized_msg.payload.clone().try_into().unwrap());
                    // let payload = from_utf8(&deserialized_msg.payload).unwrap().to_string();
                    // info!("Your new health should be {:#?}", payload);

                    // attack_event.send(AttackEvent {message: deserialized_msg.clone() });
                    let payload = isize::from_ne_bytes(deserialized_msg.payload.try_into().unwrap());
                    info!("Your new health should be {:#?}", payload);

                    // decrease health of player's monster after incoming attacks
                    let (mut player_health, mut player_stg, player_def, player_entity, player_type) = 
                    player_monster.single_mut();

                    player_health.health = payload;
                }
                else if action_type == BattleAction::Defend {
                    let payload = from_utf8(&deserialized_msg.payload).unwrap().to_string();
                    info!("Payload is {:#?}", payload);
                }
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

fn handle_monster_type_event(
    mut monster_type_event_reader: EventReader<MonsterTypeEvent>,
    mut commands: Commands,
) {
    for ev in monster_type_event_reader.iter() {
        info!("{:#?}", ev.message);
        let mut payload = usize::from_ne_bytes(ev.message.payload.clone().try_into().unwrap());

        // Create structs for opponent's monster
        let enemy_monster_stats = MonsterStats {
            typing: convert_num_to_element(payload),
            lvl: Level { level: 1 },
            hp: Health {
                max_health: 100,
                health: 100,
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
            .insert_bundle(enemy_monster_stats)
            .insert(SelectedEnemyMonster);

        commands.insert_resource(ReadyToSpawnEnemy {});
    }

}

fn handle_attack_event (
    mut attack_event_reader: EventReader<MonsterTypeEvent>,
    mut commands: Commands,
    mut player_monster: Query<
    (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
    (With<SelectedMonster>)>,
    mut enemy_monster: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (Without<SelectedMonster>, With<SelectedEnemyMonster>)>
) {
    for ev in attack_event_reader.iter() {
        info!("{:#?}", ev.message);
        let payload = isize::from_ne_bytes(ev.message.payload.clone().try_into().unwrap());

        // decrease health of player's monster after incoming attacks
        let (mut player_health, mut player_stg, player_def, player_entity, player_type) = 
        player_monster.single_mut();

        player_health.health = payload;
    }
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

// pub(crate) fn send_message(message: Message) {
//     match message.action {
//         BattleAction::Attack => {
//             let payload = message.payload;
//             //info!("{:#?}", from_utf8(&payload).unwrap());
//         }
//         BattleAction::Initialize => todo!(),
//         BattleAction::MonsterStats => todo!(),
//         BattleAction::MonsterType => {
//             let payload = message.payload;
//         }
//         BattleAction::Defend => todo!(),
//         BattleAction::Heal => todo!(),
//         BattleAction::Special => todo!(),
//     }
// }

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

pub(crate) fn setup_mult_battle_stats(
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
            }),
        )
        .insert(MultPlayerHealth)
        .insert(MultBattleUIElement);

    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // health header for opponent's monster
                TextSection::new(
                    "Health:",
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

pub(crate) fn update_mult_battle_stats(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
    mut set: ParamSet<(
        Query<&mut Health, With<SelectedMonster>>,
        Query<&mut Health, With<SelectedEnemyMonster>>,
    )>,
    mut player_health_text_query: Query<
        &mut Text,
        (With<MultPlayerHealth>, Without<MultEnemyHealth>),
    >,
    mut enemy_health_text_query: Query<
        &mut Text,
        (With<MultEnemyHealth>, Without<MultPlayerHealth>),
    >,
) {
    let mut my_health = 0;
    let mut enemy_health = 0;
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

pub(crate) fn spawn_mult_enemy_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<MultCamera>)>,
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

    // why doesn't this update
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
    commands.insert_resource(EnemyMonsterSpawned {});
}

pub(crate) fn mult_key_press_handler(
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut player_monster: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (With<SelectedMonster>),
    >,
    mut enemy_monster: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (Without<SelectedMonster>, With<SelectedEnemyMonster>),
    >,
    asset_server: Res<AssetServer>,
    game_client: Res<GameClient>,
) {
    if enemy_monster.is_empty() {
        info!("Monsters are missing!");
        return;
    }

    // Get player and opponent monster data out of the query
    let (mut player_health, mut player_stg, player_def, player_entity, player_type) = 
        player_monster.single_mut();

    let (mut enemy_health, enemy_stg, enemy_def, enemy_entity, enemy_type) =
        enemy_monster.single_mut();

    if input.just_pressed(KeyCode::A) {
        // ATTACK
        info!("Attack!");

        enemy_health.health -= player_stg.atk as isize;

        let msg = Message {
            action: BattleAction::Attack,
            payload: enemy_health.health.to_ne_bytes().to_vec(),
        };
        game_client
            .socket
            .udp_socket
            .send(&bincode::serialize(&msg).unwrap());

    } else if input.just_pressed(KeyCode::Q) {
        // ABORT
        info!("Quit!")
    } else if input.just_pressed(KeyCode::D) {
        // DEFEND
        info!("Defend!");

        let msg = Message {
            action: BattleAction::Defend,
            payload: "Sent defend message".to_string().into_bytes(),
        };
        game_client
            .socket
            .udp_socket
            .send(&bincode::serialize(&msg).unwrap());

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
