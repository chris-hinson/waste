use crate::backgrounds::{Background, LEVEL_HEIGHT, LEVEL_WIDTH, TILE_SIZE, WIN_H, WIN_W};
use bevy::prelude::*;

// original 8px/frame movement equalled 480 px/sec.
// frame-independent movement is in px/second (480 px/sec.)
pub(crate) const PLAYER_SPEED: f32 = 480.;

// We'll wanna replace these with animated sprite sheets later
pub(crate) const PLAYER_SPRITE: &str = "characters/stickdude.png";

#[derive(Component)]
pub(crate) struct Player;

// Taken from Dr. Farnan's examples at
// https://github.com/nfarnan/cs1666_examples/blob/main/bevy/examples/bv07_side_scroll.rs
//
// This will need to be edited heavily, of course, in order to make it so that the movement
// is actually appropriate for our game.
pub(crate) fn move_player(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player: Query<&mut Transform, (With<Player>, Without<Background>)>,
) {
    if player.is_empty() {
        error!("Couldn't find a player to move...");
        return;
    }
    // PLAYER_MOVEMENT = pixels/second = pixels/frame * frames/second
    let PLAYER_MOVEMENT = PLAYER_SPEED * time.delta_seconds();
    let mut pt = player.single_mut();

    let mut x_vel = 0.;
    let mut y_vel = 0.;

    if input.pressed(KeyCode::W) {
        y_vel += PLAYER_MOVEMENT;
        x_vel = 0.;
    }

    if input.pressed(KeyCode::S) {
        y_vel -= PLAYER_MOVEMENT;
        x_vel = 0.;
    }

    if input.pressed(KeyCode::A) {
        x_vel -= PLAYER_MOVEMENT;
        y_vel = 0.;
    }

    if input.pressed(KeyCode::D) {
        x_vel += PLAYER_MOVEMENT;
        y_vel = 0.;
    }

    // Most of these numbers come from debugging
    // and seeing what works.
    pt.translation.x = if pt.translation.x + x_vel > LEVEL_WIDTH - (WIN_W / 2. + TILE_SIZE / 4.) {
        LEVEL_WIDTH - (WIN_W / 2. + TILE_SIZE / 4.)
    } else if pt.translation.x + x_vel <= (-WIN_W / 2. + TILE_SIZE / 4.) {
        -WIN_W / 2. + TILE_SIZE / 4.
    } else {
        pt.translation.x + x_vel
    };

    pt.translation.y = if pt.translation.y + y_vel > LEVEL_HEIGHT - (WIN_H / 2. + TILE_SIZE / 2.) {
        LEVEL_HEIGHT - (WIN_H / 2. + TILE_SIZE / 2.)
    } else if pt.translation.y + y_vel <= (-WIN_H / 2. + TILE_SIZE / 2.) {
        -WIN_H / 2. + TILE_SIZE / 2.
    } else {
        pt.translation.y + y_vel
    };
}
