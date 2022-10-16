use bevy::{prelude::*};
use crate::player::{Player, Velocity};
use crate::backgrounds::{Background, LEVEL_HEIGHT, LEVEL_WIDTH};
use crate::{WIN_H, WIN_W};

#[derive(Component)]
pub(crate) struct MainCamera;

#[derive(Component)]
pub(crate) struct MenuCamera;

#[derive(Component)]
pub(crate) struct SlidesCamera;

pub(crate) fn move_camera(
    player: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
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

    let (pt, _pv) = player.single();
    let ct = camera.single();

    let x = if pt.translation.x < 0. {
        0.
    } else if pt.translation.x > (LEVEL_WIDTH - WIN_W) {
        LEVEL_WIDTH - WIN_W
    } else if pt.translation.x > (ct.translation.x + WIN_W/2.) {
        ct.translation.x + WIN_W
    } else if pt.translation.x < (ct.translation.x - WIN_W/2.) {
        ct.translation.x - WIN_W
    } else {
        ct.translation.x
    };

    let y = if pt.translation.y < 0.{
        0.
    } else if pt.translation.y > (LEVEL_HEIGHT - WIN_H) {
        LEVEL_HEIGHT - WIN_H
    } else if pt.translation.y > (ct.translation.y + WIN_H/2.) {
        ct.translation.y + WIN_H
    } else if pt.translation.y < (ct.translation.y - WIN_H/2.) {
        ct.translation.y - WIN_H
    } else {
        ct.translation.y
    };

    // Move camera only when the player actually leaves a screen boundary
     
    for mut transform in camera.iter_mut() {
        *transform = Transform::from_xyz(x, y, 0.);
    }
}
