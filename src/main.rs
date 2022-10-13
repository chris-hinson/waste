use bevy::{prelude::*, window::PresentMode};
use std::convert::From;

// GAMEWIDE CONSTANTS
pub(crate) const TITLE: &str = "Waste";
pub(crate) const WIN_W: f32 = 1280.;
pub(crate) const WIN_H: f32 = 720. ;
pub(crate) const PLAYER_SPEED: f32 = 500.;
pub(crate) const ACCEL_RATE: f32 = 100.;
pub(crate) const TILE_SIZE: f32 = 64.;

// CUSTOM MODULE DEFINITIONS AND IMPORTS

// Credit slides and systems
mod credits;
use credits::*;

// Backgrounds and systems to scroll them
mod backgrounds;
use backgrounds::*;

// Player and systems
mod player;
use player::*;

// Camera related movement
mod camera;
use camera::*;

mod wfc;
use wfc::*;

// END CUSTOM MODULES

#[derive(Component)]
struct Tile;


fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from(TITLE),
            width: WIN_W,
            height: WIN_H,
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(show_slide)
        .add_system(move_player)
        .add_system(move_camera)

        .run();
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // info!("Printing credits...");
    commands.spawn_bundle(Camera2dBundle::default());

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
    let starting_chuck = wfc(WIN_H as usize, WIN_W as usize);
    // info!("{:?}", starting_chuck);

    let map_handle = asset_server.load("overworld_tilesheet.png");
	// let map_handle = asset_server.load("overworld_tilesheet.png");
	let map_atlas = TextureAtlas::from_grid(map_handle, 
		Vec2::splat(TILE_SIZE), 7, 6);

	let map_atlas_len = map_atlas.textures.len();
	let map_atlas_handle = texture_atlases.add(map_atlas.clone());

	println!("Number of texture atlases: {}", map_atlas_len);

	// from center of the screen to half a tile from edge
	// so the tile will never be "cut in half" by edge of screen
	let x_bound = WIN_W/2. - TILE_SIZE/2.;
	let y_bound = WIN_H/2. - TILE_SIZE/2.;
	let mut x = -x_bound;
	let mut y = y_bound;

	for i in 0..starting_chuck.len(){
        for j in 0..starting_chuck[i].len(){
            let t = Vec3::new(
                x,
                y,
                0.,
            );
            commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: map_atlas_handle.clone(),
                transform: Transform {
                    translation: t,
                    ..default()
                },
                sprite: TextureAtlasSprite {
                    index: starting_chuck[i][j],
                    ..default()
                },
                ..default()
            })
            .insert(Tile);
                // break;
            x += TILE_SIZE;
            if x > x_bound {
                x = -x_bound;
                y -=  TILE_SIZE;
            }
    
        }
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
