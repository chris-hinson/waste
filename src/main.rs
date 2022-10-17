use bevy::{prelude::*, window::PresentMode};
use std::convert::From;

// GAMEWIDE CONSTANTS
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub(crate) enum GameState {
    Start,
    Pause,
    Playing,
    Credits,
}

pub(crate) const TITLE: &str = "Waste";
// END GAMEWIDE CONSTANTS

// CUSTOM MODULE DEFINITIONS AND IMPORTS
//mod statements:
mod backgrounds;
mod camera;
mod credits;
mod player;
mod start_menu;
mod wfc;

//use statements:
use backgrounds::*;
use camera::*;
use credits::*;
use player::*;
use start_menu::*;
use wfc::*;

// END CUSTOM MODULES

fn main() {
    App::new()
        //Starts game at main menu
        .add_state(GameState::Start)
        .insert_resource(WindowDescriptor {
            title: String::from(TITLE),
            width: WIN_W,
            height: WIN_H,
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        //adds MainMenu
        .add_plugin(MainMenuPlugin)
        .add_plugin(CreditsPlugin) // Must find a way to conditionally set up plugins
        //.add_startup_system(setup)
        .add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(init_background)
                .with_system(setup_game),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(move_player)
                .with_system(move_camera),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(despawn_game))
        .add_system(bevy::window::close_on_esc)
        // .add_system(move_player)
        // .add_system(move_camera)
        .run();
}

pub(crate) fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<
        Entity,
        (
            With<Camera2d>,
            Without<MainCamera>,
            Without<Player>,
            Without<Background>,
        ),
    >,
) {
    // Despawn other cameras
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // done so that this camera doesn't mess with any UI cameras for start or pause menu
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MainCamera);

    // Draw the player
    // He's so smol right now
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(PLAYER_SPRITE),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        // Was considering giving player marker struct an xyz component
        // til I realized transform handles that for us.
        .insert(Player);
}

pub(crate) fn despawn_game(
    mut commands: Commands,
    camera_query: Query<Entity, With<MainCamera>>,
    background_query: Query<Entity, With<Background>>,
    player_query: Query<Entity, With<Player>>,
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
}
