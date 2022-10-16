use bevy::{prelude::*};
use crate::player::{Player};
use crate::wfc::wfc;

pub(crate) const TILE_SIZE: f32    = 64.  ;
pub(crate) const MAP_WIDTH: usize  = 20   ;
pub(crate) const MAP_HEIGHT: usize = 12   ;
pub(crate) const WIN_H: f32        = 720. ;
pub(crate) const WIN_W: f32        = 1280.;
pub(crate) const LEVEL_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;
pub(crate) const LEVEL_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE;


#[derive(Component)]
pub(crate) struct Background;

pub(crate) fn init_background(mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {

    let starting_chuck = wfc();
    // info!("{:?}", starting_chuck);

    let map_handle = asset_server.load("backgrounds/overworld_tilesheet.png");
    let map_atlas = TextureAtlas::from_grid(map_handle, 
        Vec2::splat(TILE_SIZE), 7, 6);

    let map_atlas_len = map_atlas.textures.len();
    let map_atlas_handle = texture_atlases.add(map_atlas.clone());

    println!("Number of texture atlases: {}", map_atlas_len);

    // from center of the screen to half a tile from edge
    // so the tile will never be "cut in half" by edge of screen
    let x_bound = WIN_W - TILE_SIZE/2.;
    let y_bound = WIN_H - TILE_SIZE/2.;
    let mut x = -x_bound;
    let mut y = y_bound;

    for i in 0..starting_chuck.len(){
        for j in 0..starting_chuck[i].len(){
            let tile = starting_chuck[i][j];
            let t = Vec3::new(x, y, 0.,);
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
            .insert(Background);
                // break;
            x += TILE_SIZE;
            if x > x_bound {
                x = -x_bound;
                y -=  TILE_SIZE;
            }

        }
    }

}





