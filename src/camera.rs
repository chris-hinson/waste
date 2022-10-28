use std::thread::current;

use bevy::{prelude::*};
use crate::player::{Player};
use crate::backgrounds::{
    Tile,
    LEVEL_HEIGHT, LEVEL_WIDTH, 
    WIN_H, WIN_W,
    TILE_SIZE,
    Chunk, ChunkCenter,
    // CHUNK_HEIGHT, CHUNK_WIDTH
};

#[derive(Component)]
pub(crate) struct MainCamera;

#[derive(Component)]
pub(crate) struct MenuCamera;

#[derive(Component)]
pub(crate) struct SlidesCamera;

pub(crate) const CAMERA_Z_VALUE: f32 = 100.;

// #[derive(Component)]
// pub(crate) struct BattleCamera;

pub(crate) fn move_camera(
    mut player: Query<&mut Player>,
    pt_query: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera2d>, With<MainCamera>, Without<Player>, Without<Tile>)>,
) {
    if camera.is_empty() {
        error!("Found no camera...?");
        return;
    }
    if player.is_empty() {
        error!("Found no player...?");
        return;
    }
    if pt_query.is_empty() {
        error!("Found no player position...?");
        return;
    }

    // let pt = player.single().1;
    let mut pr = player.single_mut();
    let ct = camera.single();
    let pt = pt_query.single();


    let x = if pt.translation.x > (ct.translation.x + WIN_W/2.) {
        ct.translation.x + WIN_W
    } else if pt.translation.x < (ct.translation.x - WIN_W/2.) {
        ct.translation.x - WIN_W
    } else {
        ct.translation.x
    };

    // Change the player's current chunk center if the camera has switched chunks
    if pt.translation.x > (ct.translation.x + WIN_W/2.) {
        pr.current_chunk = ChunkCenter {
            transform: Transform::from_xyz(
                pr.current_chunk.transform.translation.x + WIN_W, 
                pr.current_chunk.transform.translation.y,
                pr.current_chunk.transform.translation.z
            ),
        };
    } else if pt.translation.x < (ct.translation.x - WIN_W/2.) {
        pr.current_chunk = ChunkCenter {
            transform: Transform::from_xyz(
                pr.current_chunk.transform.translation.x - WIN_W, 
                pr.current_chunk.transform.translation.y,
                pr.current_chunk.transform.translation.z
            ),
        };
    }

    if pt.translation.y > (ct.translation.y + WIN_H/2. + TILE_SIZE) {
        pr.current_chunk = ChunkCenter {
            transform: Transform::from_xyz(
                pr.current_chunk.transform.translation.x, 
                pr.current_chunk.transform.translation.y + WIN_H,
                pr.current_chunk.transform.translation.z
            ),
        };
    } else if pt.translation.y < (ct.translation.y - WIN_H/2. - TILE_SIZE) {
        pr.current_chunk = ChunkCenter {
            transform: Transform::from_xyz(
                pr.current_chunk.transform.translation.x, 
                pr.current_chunk.transform.translation.y - WIN_H,
                pr.current_chunk.transform.translation.z
            ),
        };
    } 

    // I am not sure why the window top is slightly off balance
    // and needing the +/- TILE_SIZE check, but it prevents halves of tiles
    // compounding and resulting in mostly void screens.
    let y = if pt.translation.y > (ct.translation.y + WIN_H/2. + TILE_SIZE) {
        ct.translation.y + WIN_H
    } else if pt.translation.y < (ct.translation.y - WIN_H/2. - TILE_SIZE) {
        ct.translation.y - WIN_H
    } else {
        ct.translation.y
    };

    // Move camera only when the player actually leaves a screen boundary
     
    for mut transform in camera.iter_mut() {
        *transform = Transform::from_xyz(x, y, CAMERA_Z_VALUE);
    }
}
