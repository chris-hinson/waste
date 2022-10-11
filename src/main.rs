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


#[derive(Component)]
pub struct MainCamera;

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
        .add_startup_system(setup)
        .add_system(show_slide)
        .add_system(move_player)
        .add_system(move_camera)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // done so that this camera doesn't mess with any UI cameras for start or pause menu
	let mut camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MainCamera);


    // TODO: What do we do to this so that it is only 
    // displaying these slides when a menu button is pressed to go to credits?
    // let slides = vec![
    //     "credits/gavin_credit.png",
    //     "credits/dan_credit.png",
    //     "credits/camryn_credit.png",
    //     "credits/caela_credit.png",
    //     "credits/prateek_credit.png",
    //     "credits/chase_credit.png",
    //     "credits/nathan_credit.png",
    //     "credits/chris_credit.png",
    // ];

    // for i in 0..slides.len() {
    //     commands.spawn_bundle(SpriteBundle {
    //         texture: asset_server.load(slides[i]),
    //         visibility: Visibility {
    //             is_visible: if i == 0 { true } else { false },
    //         },
    //         transform: Transform::from_xyz(0., 0., 0.),
    //         ..default()
    //     });
    // }

    // commands.spawn().insert(SlideTimer {
    //     timer: Timer::from_seconds(5.0, true),
    // });
    // commands.spawn().insert(SlideDeck {
    //     total_slides: slides.len(),
    //     current_slide: 1,
    // });
    
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
