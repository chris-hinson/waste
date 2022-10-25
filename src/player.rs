use bevy::{prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::state::NextState;
use crate::GameState;
use crate::backgrounds::{Tile, TILE_SIZE, LEVEL_WIDTH, LEVEL_HEIGHT, WIN_H, WIN_W, MonsterTile};

// original 8px/frame movement equalled 480 px/sec.
// frame-independent movement is in px/second (480 px/sec.)
pub(crate) const PLAYER_SPEED: f32 = 480.;
// We'll wanna replace these with animated sprite sheets later
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
		for (mut sprite, _) in player.iter_mut() {
			sprite.index = 0;
		}
	}
	else if input.just_released(KeyCode::D) {
		for (mut sprite, _) in player.iter_mut() {
			sprite.index = ANIM_FRAMES
		}
	}
	else if input.just_released(KeyCode::A) {
		for (mut sprite, _) in player.iter_mut() {
			sprite.index = ANIM_FRAMES * 2
		}
	}
	else if input.just_released(KeyCode::W) {
		for (mut sprite, _) in player.iter_mut() {
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

pub(crate) fn move_player(
	input: Res<Input<KeyCode>>,
  	time: Res<Time>,
	mut commands: Commands,
	mut player: Query<&mut Transform, (With<Player>, Without<Tile>)>,
	monster_tiles: Query<&mut MonsterTile>,
){
	if player.is_empty() {
		error!("Couldn't find a player to move...");
		return;
	}

    // PLAYER_MOVEMENT = pixels/second = pixels/frame * frames/second
    let player_movement = PLAYER_SPEED * time.delta_seconds();
	let mut pt = player.single_mut();

	let mut x_vel = 0.;
	let mut y_vel = 0.;

	if input.pressed(KeyCode::W) {
		y_vel += player_movement;
		x_vel = 0.;
	}

	if input.pressed(KeyCode::S) {
		y_vel -= player_movement;
		x_vel = 0.;
	}

	if input.pressed(KeyCode::A) {
		x_vel -= player_movement;
		y_vel = 0.;
	}

	if input.pressed(KeyCode::D) {
		x_vel += player_movement;
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
		let mt_position = monster_tile.transform.translation;
		let collision = collide(pt.translation, Vec2::splat(32.), mt_position, Vec2::splat(32.));
		match collision {
			None => {},
			Some(_) => {
				// temporary marker
				//println!("Collided with monster! Battle!");
				// switches from Playing -> Battle state
				commands.insert_resource(NextState(GameState::Battle));
			}
		}
	}
}
