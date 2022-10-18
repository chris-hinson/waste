use bevy::{prelude::*, sprite::collide_aabb::collide, sprite::collide_aabb::Collision};
use crate::backgrounds::{Tile, TILE_SIZE, LEVEL_WIDTH, LEVEL_HEIGHT, WIN_H, WIN_W, MonsterTile};
pub(crate) const PLAYER_SPEED: f32 = 4.;
pub(crate) const ANIM_TIME: f32 = 0.15;
pub(crate) const ANIM_FRAMES: usize = 4;

#[derive(Component)]
pub(crate) struct Player;

#[derive(Component, Deref, DerefMut)]
pub(crate) struct AnimationTimer(pub(crate) Timer);


pub(crate) fn animate_sprite(
    time: Res<Time>,
	input: Res<Input<KeyCode>>,
	mut player: Query<(&mut TextureAtlasSprite, &mut AnimationTimer), With<Player>>,
) {

	if input.just_released(KeyCode::S) {
		for (mut sprite, mut timer) in player.iter_mut() {
			sprite.index = 0;
		}
	}
	else if input.just_released(KeyCode::D) {
		for (mut sprite, mut timer) in player.iter_mut() {
			sprite.index = ANIM_FRAMES
		}
	}
	else if input.just_released(KeyCode::A) {
		for (mut sprite, mut timer) in player.iter_mut() {
			sprite.index = ANIM_FRAMES * 2
		}
	}
	else if input.just_released(KeyCode::W) {
		for (mut sprite, mut timer) in player.iter_mut() {
			sprite.index = ANIM_FRAMES * 3;
		}
	}

	if input.pressed(KeyCode::S){
		for (mut sprite, mut timer) in player.iter_mut() {
			timer.tick(time.delta());
			if timer.just_finished() {
				// let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
				sprite.index = (sprite.index + 1) % ANIM_FRAMES;
			}
		}
	}
	else if input.pressed(KeyCode::D){
		for (mut sprite, mut timer) in player.iter_mut() {
			timer.tick(time.delta());
			if timer.just_finished() {
				sprite.index = ((sprite.index + 1) % ANIM_FRAMES) + 4;
			}
		}
	}
	else if input.pressed(KeyCode::A){
		for (mut sprite, mut timer) in player.iter_mut() {
			timer.tick(time.delta());
			if timer.just_finished() {
				sprite.index = ((sprite.index + 1) % ANIM_FRAMES) + 8;
			}
		}
	}
	else if input.pressed(KeyCode::W){
		for (mut sprite, mut timer) in player.iter_mut() {
			timer.tick(time.delta());
			if timer.just_finished() {
				sprite.index = ((sprite.index + 1) % ANIM_FRAMES) + 12;
			}
		}
	}
}

// Taken from Dr. Farnan's examples at
// https://github.com/nfarnan/cs1666_examples/blob/main/bevy/examples/bv07_side_scroll.rs
//
// This will need to be edited heavily, of course, in order to make it so that the movement
// is actually appropriate for our game.
pub(crate) fn move_player(
	input: Res<Input<KeyCode>>,
	mut player: Query<&mut Transform, (With<Player>, Without<Tile>)>,
	mut monster_tiles: Query<&mut MonsterTile>,
){
	if player.is_empty() {
		error!("Couldn't find a player to move...");
		return;
	}

	let mut pt = player.single_mut();

	let mut x_vel = 0.;
	let mut y_vel = 0.;

	if input.pressed(KeyCode::W) {
		y_vel += PLAYER_SPEED;
		x_vel = 0.;
	}

	if input.pressed(KeyCode::S) {
		y_vel -= PLAYER_SPEED;
		x_vel = 0.;
	}

	if input.pressed(KeyCode::A) {
		x_vel -= PLAYER_SPEED;
		y_vel = 0.;
	}

	if input.pressed(KeyCode::D) {
		x_vel += PLAYER_SPEED;
		y_vel = 0.;
	}

	// Most of these numbers come from debugging
	// and seeing what works. 
	pt.translation.x = if pt.translation.x + x_vel > LEVEL_WIDTH - (WIN_W/2. + TILE_SIZE/4.){
		LEVEL_WIDTH - (WIN_W/2. + TILE_SIZE/4.)
	} else if pt.translation.x + x_vel <= (-WIN_W/2. + TILE_SIZE/4.) {
		-WIN_W/2. + TILE_SIZE/4.
	} else {
		pt.translation.x + x_vel
	};

	pt.translation.y = if pt.translation.y + y_vel > LEVEL_HEIGHT - (WIN_H/2. + TILE_SIZE/2.) {
		LEVEL_HEIGHT - (WIN_H/2. + TILE_SIZE/2.)
	} else if pt.translation.y + y_vel <= (-WIN_H/2. + TILE_SIZE/2.) {
		-WIN_H/2. + TILE_SIZE/2.
	} else {
		pt.translation.y + y_vel
	};

	// This is where we will check for collisions with monsters
	for monster_tile in monster_tiles.iter() {
		let monster_pos = monster_tile.transform.translation;
		let collision = collide(pt.translation, Vec2::splat(64.), monster_pos, Vec2::splat(64.));
		match collision {
			None => {},
			Some(_) => {
				// Now as long as the player is standing on a tile, this will keep triggering
				// This looks like an easy fix with state transitioning, we will see when that's implemented
				// If not the solution is also simple: we kick the player out of the monster tile :)
				println!("Collided with monster! Battle!");
			}
		}
	}
}
