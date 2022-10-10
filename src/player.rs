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

// Taken from Dr. Farnan's examples at
// https://github.com/nfarnan/cs1666_examples/blob/main/bevy/examples/bv07_side_scroll.rs
//
// This will need to be edited heavily, of course, in order to make it so that the movement
// is actually appropriate for our game.
pub(crate) fn move_player(
	time: Res<Time>,
	input: Res<Input<KeyCode>>,
	mut player: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
){
    // Bail if no player has been drawn.
    if player.is_empty() {
        return;
    }


	let (mut pt, mut pv) = player.single_mut();

	let mut deltav = Vec2::splat(0.);

	if input.pressed(KeyCode::A) {
		deltav.x -= 1.;
	}

	if input.pressed(KeyCode::D) {
		deltav.x += 1.;
	}

	if input.pressed(KeyCode::W) {
		deltav.y += 1.;
	}

	if input.pressed(KeyCode::S) {
		deltav.y -= 1.;
	}

	let deltat = time.delta_seconds();
	let acc = ACCEL_RATE * deltat;

	pv.velocity = if deltav.length() > 0. {
		(pv.velocity + (deltav.normalize_or_zero() * acc)).clamp_length_max(PLAYER_SPEED)
	}
	else if pv.velocity.length() > acc {
		pv.velocity + (pv.velocity.normalize_or_zero() * -acc)
	}
	else {
		Vec2::splat(0.)
	};
	let change = pv.velocity * deltat;

	let new_pos = pt.translation + Vec3::new(
		change.x,
		0.,
		0.,
	);
	if new_pos.x >= -(WIN_W/2.) + TILE_SIZE/2.
		&& new_pos.x <= LEVEL_LENGTH - (WIN_W/2. + TILE_SIZE/2.)
	{
		pt.translation = new_pos;
	}

	let new_pos = pt.translation + Vec3::new(
		0.,
		change.y,
		0.,
	);
	if new_pos.y >= -(WIN_H/2.) + (TILE_SIZE * 1.5)
		&& new_pos.y <= LEVEL_HEIGHT - (WIN_H/2. + TILE_SIZE/2.)
	{
		pt.translation = new_pos;
	}
}
