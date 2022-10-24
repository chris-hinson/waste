use bevy::{prelude::*};
use crate::player::{Player};
use crate::wfc::wfc;

pub(crate) const TILE_SIZE: f32      = 64.  ;
pub(crate) const MAP_WIDTH: usize    = 40   ;
pub(crate) const MAP_HEIGHT: usize   = 24   ;
// pub(crate) const CHUNK_WIDTH: usize  = 20   ;
// pub(crate) const CHUNK_HEIGHT: usize = 12   ;
pub(crate) const WIN_H: f32          = 720. ;
pub(crate) const WIN_W: f32          = 1280.;
pub(crate) const LEVEL_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;
pub(crate) const LEVEL_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE;
const DRAW_START_X: f32 = -WIN_W/2. + TILE_SIZE/2.;
const DRAW_START_Y: f32 = -WIN_H/2. + TILE_SIZE/2.;
// const DRAW_STOP_X: f32  = LEVEL_WIDTH - TILE_SIZE/2.;
// const DRAW_STOP_Y: f32  = LEVEL_HEIGHT - TILE_SIZE/2.;

pub(crate) const GAME_BACKGROUND: &str = "backgrounds/test_scroll_background.png";
pub(crate) const OVERWORLD_TILESHEET: &str = "backgrounds/overworld_tilesheet.png";

#[derive(Component)]
pub(crate) struct Tile;

#[derive(Component)]
pub(crate) struct MonsterTile {
    pub(crate) transform: Transform,
}

pub(crate) fn init_background(mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {

    let starting_chunk = wfc();
    // info!("{:?}", starting_chuck);

    let map_handle = asset_server.load(OVERWORLD_TILESHEET);
    let map_atlas = TextureAtlas::from_grid(map_handle, 
        Vec2::splat(TILE_SIZE), 7, 6);

    let map_atlas_len = map_atlas.textures.len();
    let map_atlas_handle = texture_atlases.add(map_atlas.clone());

    println!("Number of texture atlases: {}", map_atlas_len);

    // from center of the screen to half a tile from edge
    // so the tile will never be "cut in half" by edge of screen
    let mut x = DRAW_START_X;
    let mut y = DRAW_START_Y;

    for i in 0..starting_chunk.len(){
        for j in 0..starting_chunk[i].len(){
            let tile = starting_chunk[i][j];
            let t = Vec3::new(x, y, -1.,);
            commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: map_atlas_handle.clone(),
                transform: Transform {
                    translation: t,
                    ..default()
                },
                sprite: TextureAtlasSprite {
                    index: tile,
                    ..default()
                },
                ..default()
            })
            .insert(Tile);
            if tile == 4 {
                commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: map_atlas_handle.clone(),
                    transform: Transform {
                        translation: t,
                        ..default()
                    },
                    sprite: TextureAtlasSprite {
                        index: 4,
                        ..default()
                    },
                    ..default()
                })
                .insert(MonsterTile{transform: Transform::from_xyz(x, y, -1.)});
            }

            x += TILE_SIZE;
            // if x > x_bound {
            //     x = -WIN_W/2. + TILE_SIZE/2.;
            //     y =  TILE_SIZE;
            // }
        }
        x = DRAW_START_X;
        y += TILE_SIZE;
    }

}

// OLD BACKGROUND CODE
// // Draw a bunch of backgrounds
// let mut x_offset = 0.;
// // Don't worry about the misalignment, it isn't that this code is wrong
// // it's that my drawing is a horrible mishapen creature.
// while x_offset < LEVEL_WIDTH {
//     commands
//         .spawn_bundle(SpriteBundle {
//             texture: asset_server.load(GAME_BACKGROUND),
//             transform: Transform::from_xyz(x_offset, 0., 0.),
//             ..default()
//         })
//         .insert(Background);

//         // Now do all the backgrounds above it.
//         let mut y_offset = WIN_H;
//         while y_offset < LEVEL_HEIGHT {
//             commands
//                 .spawn_bundle(SpriteBundle {
//                     texture: asset_server.load(GAME_BACKGROUND),
//                     transform: Transform::from_xyz(x_offset, y_offset, 0.),
//                     ..default()
//                 })
//                 .insert(Background);
//             y_offset += WIN_H;
//         }
//     x_offset += WIN_W;
// }
