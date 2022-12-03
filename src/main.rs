// Warn about poor coding practices resulting in massive code bloat
#![warn(unused)]
// Deny poor coding practices with quiet negative consequences
#![deny(unsafe_code)]
#![deny(unreachable_code)]
#![deny(while_true)]
#![deny(where_clauses_object_safety)]
#![deny(private_in_public)]
// Deny only by clippy
#![deny(clippy::empty_loop)]
#![deny(clippy::while_immutable_condition)]
#![deny(clippy::self_assignment)]
use bevy::{prelude::*, window::PresentMode};

use iyes_loopless::prelude::*;
use std::convert::From;

// GAMEWIDE CONSTANTS
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub(crate) enum GameState {
    Start,
    Pause,
    StartPlaying,
    Playing,
    Battle,
    Credits,
    Help,
    MultiplayerMenu,
    MultiplayerWaiting,
    MultiplayerPvPBattle,
    MultiplayerPvEBattle,
}

pub(crate) const TITLE: &str = "Waste";
// END GAMEWIDE CONSTANTS

// CUSTOM MODULE DEFINITIONS AND IMPORTS
//mod statements:
mod backgrounds;
mod battle;
mod camera;
mod credits;
mod game_client;
mod help;
mod monster;
mod multiplayer_menu;
mod multiplayer_pve;
mod multiplayer_pvp;
mod multiplayer_waiting;
mod networking;
mod pause;
mod player;
mod quests;
mod start_menu;
mod wfc;
mod world;

//use statements:
use backgrounds::*;
use battle::*;
use camera::*;
use credits::*;
use game_client::*;
use help::*;
use monster::*;
use multiplayer_menu::*;
use multiplayer_pve::*;
use multiplayer_pvp::*;
use multiplayer_waiting::*;
use networking::*;
use pause::*;
use player::*;
use quests::*;
use start_menu::*;
use wfc::*;
use world::*;

// END CUSTOM MODULES

#[derive(Debug, Clone, Component)]
pub struct UIText;

#[derive(Debug, Clone, Component)]
pub struct TextTimer {
    pub time: Timer,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from(TITLE),
            width: WIN_W,
            height: WIN_H,
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .init_resource::<WorldMap>()
        .init_resource::<GameProgress>()
        .init_resource::<TypeSystem>()
        .init_resource::<ProcGen>()
        .init_resource::<MultiplayerModeSelected>()
        .init_resource::<TextBuffer>()
        .init_resource::<GameClientNotInitialized>()
        .add_plugins(DefaultPlugins)
        // Starts game at main menu
        // Initial state should be "loopless"
        .add_loopless_state(GameState::Start)
        .add_plugin(MainMenuPlugin)
        .add_plugin(CreditsPlugin)
        .add_plugin(HelpPlugin)
        .add_plugin(PausePlugin)
        .add_plugin(BattlePlugin)
        .add_plugin(MultMenuPlugin)
        .add_plugin(MultiplayerWaitingPlugin)
        .add_plugin(MultPvPPlugin)
        .add_plugin(MultPvEPlugin)
        .add_enter_system_set(
            GameState::StartPlaying,
            // This system set is unconditional, as it is being added in an enter helper
            SystemSet::new()
                .with_system(init_background)
                .with_system(setup_game),
        )
        .add_system_set(
            ConditionSet::new()
                // These systems will only run in the condition that the game is in the state
                // Playing
                .run_in_state(GameState::Playing)
                .with_system(move_player)
                .with_system(move_camera)
                .with_system(animate_sprite)
                .with_system(expand_map)
                .with_system(win_game)
                .with_system(handle_pause)
                .into(),
        )
        .add_system(display_text)
        .add_system(despawn_text)
        .run();
}

pub(crate) fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    cameras: Query<
        Entity,
        (
            With<Camera2d>,
            Without<MainCamera>,
            Without<Player>,
            Without<Tile>,
        ),
    >,
    mut game_progress: ResMut<GameProgress>,
) {
    // Despawn other cameras
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // done so that this camera doesn't mess with any UI cameras for start or pause menu
    let camera = Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 1000.),
        ..default()
    };
    commands.spawn_bundle(camera).insert(MainCamera);

    let texture_handle = asset_server.load("characters/sprite_movement.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    // Draw the player
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        // Was considering giving player marker struct an xyz component
        // til I realized transform handles that for us.
        .insert(AnimationTimer(Timer::from_seconds(ANIM_TIME, true)))
        //player stats init here:
        .insert(Player {
            current_chunk: (0, 0),
            //constants can be found in player.rs,
        });

    // Give the player a monster
    let initial_monster_stats = MonsterStats {
        ..Default::default()
    };
    let initial_monster = commands
        .spawn()
        .insert_bundle(initial_monster_stats)
        .insert(SelectedMonster)
        .insert(PartyMonster)
        .id();
    // initial_monster.insert(SelectedMonster);
    game_progress.new_monster(initial_monster, initial_monster_stats);

    // Finally, transition to normal playing state
    commands.insert_resource(NextState(GameState::Playing));
}

/// Tear down ALL significant resources for the game, and despawn all relevant
/// in game entities. This should be used when bailing out of the credits state
/// after beating the game, or when exiting multiplayer to move to singleplayer.*
///
/// *Multiplayer may need to add their own relevant resources or queries to despawn
pub(crate) fn teardown(
    mut commands: Commands,
    camera_query: Query<Entity, With<MainCamera>>,
    background_query: Query<Entity, With<Tile>>,
    player_query: Query<Entity, With<Player>>,
    monster_query: Query<Entity, With<PartyMonster>>,
    npc_query: Query<Entity, With<NPC>>,
) {
    // Despawn main camera
    camera_query.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // Despawn world
    background_query.for_each(|background| {
        commands.entity(background).despawn();
    });

    // Despawn player
    player_query.for_each(|player| {
        commands.entity(player).despawn();
    });

    // Despawn monsters
    monster_query.for_each(|monster| {
        commands.entity(monster).despawn();
    });

    // Despawn NPCs
    npc_query.for_each(|npc| {
        commands.entity(npc).despawn();
    });

    // Remove the game client, as we will reinitialize it on
    // next setup
    commands.remove_resource::<GameClient>();
    // Remove the old worldmap
    commands.remove_resource::<WorldMap>();
    // Remove the game progress resource
    commands.remove_resource::<GameProgress>();
    // Re-initialize the resources
    commands.init_resource::<WorldMap>();
    commands.init_resource::<GameProgress>();
}

/// Mark that game has been completed and transition to credits.
pub(crate) fn win_game(mut commands: Commands, game_progress: ResMut<GameProgress>) {
    if game_progress.num_boss_defeated == 5 {
        commands.insert_resource(NextState(GameState::Credits));
    }
}

pub(crate) fn handle_pause(mut commands: Commands, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        // Change to pause menu state
        commands.insert_resource(NextState(GameState::Pause));
    }
}

pub fn display_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut text_buffer: ResMut<TextBuffer>,
) {
    // take the text buffer and display it on the screen
    // let text = text_buffer.bottom_middle.pop_front();
    // if text.is_none() {
    //     return;
    // }
    let display_latest = 725.0;
    for i in 0..text_buffer.bottom_text.len() {
        let mut text = text_buffer.bottom_text.get_mut(i);
        if text.as_ref().unwrap().pooled {
            continue;
        }

        text.as_mut().unwrap().pooled = true;
        commands
            .spawn_bundle(
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    TextSection::new(
                        text.as_ref().unwrap().text.clone(),
                        TextStyle {
                            font: asset_server.load("buttons/joystix monospace.ttf"),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 30.0,
                        color: Color::BLACK,
                    }),
                ])
                .with_text_alignment(TextAlignment::CENTER)
                .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(display_latest - i as f32 * 30.),
                        left: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                }),
            )
            .insert(UIText)
            .insert(TextTimer {
                time: Timer::from_seconds(2., true),
            });
    }
}

pub fn despawn_text(
    mut commands: Commands,
    mut text_timer: Query<(Entity, &mut TextTimer)>,
    time: Res<Time>,
    mut text_buffer: ResMut<TextBuffer>,
) {
    for (text_entity, mut timer) in text_timer.iter_mut() {
        timer.time.tick(time.delta());
        if timer.time.finished() {
            commands.entity(text_entity).despawn_recursive();
            text_buffer.bottom_text.pop_front();
        }
    }
}
