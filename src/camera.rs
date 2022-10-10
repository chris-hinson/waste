use bevy::{prelude::*};
use crate::player::{Player, Velocity};
use crate::backgrounds::{Background, LEVEL_HEIGHT, LEVEL_LENGTH};
use crate::{WIN_H, WIN_W};

pub(crate) fn move_camera(
    player: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
    mut camera: Query<(&mut Transform), (With<Camera2d>, Without<Player>, Without<Background>)>,
) {
    if camera.is_empty() {
        info!("Found no camera...?");
        return;
    }
    if player.is_empty() {
        info!("Found no player...?");
        return;
    }

    let (pt, pv) = player.single();

    let x = if pt.translation.x < 0. {
        0.
    } else if pt.translation.x > (LEVEL_LENGTH - WIN_W) {
        LEVEL_LENGTH - WIN_W
    } else {
        pt.translation.x
    };

    let y = if pt.translation.y < 0.{
        0.
    } else if pt.translation.y > (LEVEL_HEIGHT - WIN_H) {
        LEVEL_HEIGHT - WIN_H
    } else {
        pt.translation.y
    };

    for mut transform in camera.iter_mut() {
        *transform = Transform::from_xyz(x, y, 0.);
    }
}