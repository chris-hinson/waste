use bevy::{prelude::*, 
	window::PresentMode,
};
use std::convert::From;


// GAMEWIDE CONSTANTS
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub (crate) enum GameState{
	Start,
	Pause,
	Playing,
    Credits,
}
pub(crate) const TITLE: &str = "Waste";
pub(crate) const WIN_W: f32 = 1280.;
pub(crate) const WIN_H: f32 = 720. ;
pub(crate) const PLAYER_SPEED: f32 = 500.;
pub(crate) const ACCEL_RATE: f32 = 100.;
// END GAMEWIDE CONSTANTS

// CUSTOM MODULE DEFINITIONS AND IMPORTS
//mod statements:
mod credits;
mod backgrounds;
mod player;
mod camera;
mod start_menu;

//use statements:
use credits::*;
use backgrounds::*;
use player::*;
use camera::*;
use start_menu::*;

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
        .add_system_set(SystemSet::on_enter(GameState::Playing)
            .with_system(setup_game))
        .add_system_set(SystemSet::on_update(GameState::Playing)
            .with_system(move_player)
            .with_system(move_camera))
        .add_system_set(SystemSet::on_exit(GameState::Playing)
            .with_system(despawn_game))
        // .add_system(move_player)
        // .add_system(move_camera)
        .run();
}

pub(crate) fn setup_game(mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, (With<Camera2d>, Without<MainCamera>, Without<Player>, Without<Background>)>
) {
    // Despawn other cameras
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // done so that this camera doesn't mess with any UI cameras for start or pause menu
	let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MainCamera);
    
    // Draw a bunch of backgrounds
    let mut x_offset = 0.;
    // Don't worry about the misalignment, it isn't that this code is wrong
    // it's that my drawing is a horrible mishapen creature.
    while x_offset < LEVEL_LENGTH {
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load(GAME_BACKGROUND),
                transform: Transform::from_xyz(x_offset, 0., 0.),
                ..default()
            })
            .insert(Background);

            // Now do all the backgrounds above it.
            let mut y_offset = WIN_H;
            while y_offset < LEVEL_HEIGHT {
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load(GAME_BACKGROUND),
                        transform: Transform::from_xyz(x_offset, y_offset, 0.),
                        ..default()
                    })
                    .insert(Background);
                y_offset += WIN_H;
            }
        x_offset += WIN_W;
    }

    // Draw the player
    // He's so smol right now
    commands
        .spawn_bundle(SpriteBundle { 
            texture: asset_server.load(PLAYER_SPRITE),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        // Homie needs some velocity ong or he is not going ANYWHERE
        .insert(Velocity::new())
        // Was considering giving player marker struct an xyz component
        // til I realized transform handles that for us.
        .insert(Player);

}

pub(crate) fn despawn_game(mut commands: Commands,
	camera_query: Query<Entity,  With<MainCamera>>,
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