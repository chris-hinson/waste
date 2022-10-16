use bevy::{prelude::*};
use crate::backgrounds::{Background, TILE_SIZE, LEVEL_LENGTH, LEVEL_HEIGHT};
use crate::{ACCEL_RATE, PLAYER_SPEED, WIN_H, WIN_W};

// We'll wanna replace these with animated sprite sheets later
pub(crate) const PLAYER_SPRITE: &str = "characters/stickdude.png";

#[derive(Component)]
pub(crate) struct Player;

#[derive(Component)]
pub(crate) struct Velocity {
	pub(crate) velocity: Vec2,
}

impl Velocity {
	pub(crate) fn new() -> Self {
		Self { velocity: Vec2::splat(0.) }
	}
}

impl From<Vec2> for Velocity {
	fn from(velocity: Vec2) -> Self {
		Self { velocity }
	}
}

pub(crate) fn move_player(
	input: Res<Input<KeyCode>>,
	mut player: Query<(&mut Transform), (With<Player>, Without<Background>)>,
){
	let mut pt = player.single_mut();

	let mut x_vel = 0.;
	let mut y_vel = 0.;

	if input.pressed(KeyCode::W) {
		y_vel += 4.;
	}

	if input.pressed(KeyCode::S) {
		y_vel -= 4.;
	}

	if input.pressed(KeyCode::A) {
		x_vel -= 4.;
		y_vel = 0.;
	}

	if input.pressed(KeyCode::D) {
		x_vel += 4.;
		y_vel = 0.;
	}

	pt.translation.x += x_vel;
	pt.translation.y += y_vel;
}