use bevy::{prelude::*};
use crate::player::{Player};
use crate::backgrounds::{
    Background,
    LEVEL_HEIGHT, LEVEL_WIDTH, 
    WIN_H, WIN_W,
    TILE_SIZE,
    // CHUNK_HEIGHT, CHUNK_WIDTH
};

#[derive(Component)]
pub(crate) struct MainCamera;

#[derive(Component)]
pub(crate) struct MenuCamera;

#[derive(Component)]
pub(crate) struct SlidesCamera;

pub(crate) fn move_camera(
    player: Query<&mut Transform, (With<Player>, Without<Background>)>,
    mut camera: Query<&mut Transform, (With<Camera2d>, With<MainCamera>, Without<Player>, Without<Background>)>,
) {
    if camera.is_empty() {
        info!("Found no camera...?");
        return;
    }
    if player.is_empty() {
        info!("Found no player...?");
        return;
    }

    let pt = player.single();
    let ct = camera.single();

    let x = if pt.translation.x < 0. {
        0.
    } else if pt.translation.x > (ct.translation.x + WIN_W/2.) {
        ct.translation.x + WIN_W
    } else if pt.translation.x < (ct.translation.x - WIN_W/2.) {
        ct.translation.x - WIN_W
    } else {
        ct.translation.x
    };

    // I am not sure why the window top is slightly off balance
    // and needing the +/- TILE_SIZE check, but it prevents halves of tiles
    // compounding and resulting in mostly void screens.
    let y = if pt.translation.y < 0.{
        0.
    } else if pt.translation.y > (ct.translation.y + WIN_H/2. + TILE_SIZE) {
        ct.translation.y + WIN_H + TILE_SIZE
    } else if pt.translation.y < (ct.translation.y - WIN_H/2. - TILE_SIZE) {
        ct.translation.y - WIN_H - TILE_SIZE
    } else {
        ct.translation.y
    };

    // Move camera only when the player actually leaves a screen boundary
     
    for mut transform in camera.iter_mut() {
        *transform = Transform::from_xyz(x, y, 0.);
    }
}
