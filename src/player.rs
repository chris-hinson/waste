use std::char::MAX;

use bevy::{prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::state::NextState;
use crate::world::GameProgress;
use crate::{GameState, monster};
use crate::backgrounds::{Tile, MonsterTile, HealingTile, ChestTile};
use crate::monster::{MonsterStats, MonsterPartyBundle, Enemy, Boss, SelectedMonster, Health, Level, Strength, Defense};
use rand::*;
// original 8px/frame movement equalled 480 px/sec.
// frame-independent movement is in px/second (480 px/sec.)
pub(crate) const PLAYER_SPEED: f32 = 480.;
// We'll wanna replace these with animated sprite sheets later
pub(crate) const ANIM_TIME: f32 = 0.15;
pub(crate) const ANIM_FRAMES: usize = 4;
pub(crate) const MAX_HEALTH: u16 = 10;
pub(crate) const MAX_LEVEL: u16 = 10;



#[derive(Component)]
pub(crate) struct Player{
	pub(crate) current_chunk: (isize, isize),
}

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
	mut player: Query<&mut Transform, (With<Player>, Without<Tile>, Without<MonsterTile>)>,
	monster_tiles: Query<(Entity, &Transform), (With<MonsterTile>, Without<Player>)>,
	healing_tiles: Query<(Entity, &Transform), (With<HealingTile>, Without<Player>)>,
	chest_tiles: Query<(Entity, &Transform), (With<ChestTile>, Without<Player>)>,
	mut monster_hp: Query<&mut Health, Without<Enemy>>,
	mut game_progress: ResMut<GameProgress>,
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
	pt.translation.x = pt.translation.x + x_vel;


	pt.translation.y = pt.translation.y + y_vel;

	// This is where we will check for collisions with monsters

	// This is awful, can we do this without loops?
	for (monster_tile, tile_pos) in monster_tiles.iter() {
		let mt_position = tile_pos.translation;
		let collision = collide(pt.translation, Vec2::splat(32.), mt_position, Vec2::splat(32.));
		match collision {
			None => {},
			Some(_) => {
				// temporary marker
				//println!("Collided with monster! Battle!");
				// switches from Playing -> Battle state
				// The level_boss_awaken bool is by default false
				// it will appear after we level up(defeat 5 monsters)
				if !game_progress.level_boss_awaken {
					// Normal monster
					let enemy_stats = MonsterStats {
						lvl: Level { level: game_progress.current_level },
						hp: Health{
							health: (game_progress.current_level*10) as isize, 
							max_health: (game_progress.current_level)*10
						},
						stg: Strength{
							atk: (game_progress.current_level)*2, 
							crt: game_progress.current_level*5, 
							crt_dmg: 2
						},
						def: Defense{
							def: game_progress.current_level*1, 
							crt_res: 10
						},
						..Default::default()
					};
					let enemy_entity = commands.spawn()
						.insert_bundle(enemy_stats.clone())
						.insert(Enemy).id();
					game_progress.enemy_stats.insert(enemy_entity, enemy_stats);
				} else {
					// Boss monster
					let enemy_stats = MonsterStats {
						lvl: Level { level: game_progress.current_level },
						// So when we battle him, he has 100 hp
						hp: Health{
							health: (game_progress.current_level*25) as isize, 
							max_health: game_progress.current_level*25
						},
						stg: Strength{
							atk: (game_progress.current_level*2), 
							crt: 5+(game_progress.current_level*5), 
							crt_dmg: 2
						},
						def: Defense{
							def: (game_progress.current_level*1), 
							crt_res: 10
						},
						..Default::default()
					};
					let enemy_entity = commands.spawn()
						.insert_bundle(enemy_stats.clone())
						.insert(Boss)
						.insert(Enemy).id();
					game_progress.enemy_stats.insert(enemy_entity, enemy_stats);
				}
				commands.entity(monster_tile).remove::<MonsterTile>();
				commands.insert_resource(NextState(GameState::Battle));
			}
		}
	}

	// check for healing tiles
	for (healing_tile, tile_pos) in healing_tiles.iter() {
		let ht_position = tile_pos.translation;
		let collision = collide(pt.translation, Vec2::splat(32.), ht_position, Vec2::splat(32.));
		match collision {
			None => {},
			Some(_) => {
				// temporary marker
				for mut hp in monster_hp.iter_mut() {
					hp.health = hp.max_health as isize;
				}
				game_progress.num_living_monsters = game_progress.num_monsters;
				info!("Monster health restored.");
				commands.entity(healing_tile).remove::<HealingTile>();
			}
		}
	}

	// check for chest tiles
	for (chest_tile, tile_pos) in chest_tiles.iter() {
		let ht_position = tile_pos.translation;
		let collision = collide(pt.translation, Vec2::splat(32.), ht_position, Vec2::splat(32.));
		match collision {
			None => {},
			Some(_) => {
				let item = rand::thread_rng().gen_range(0..=1) as usize;
				let item_got = match item {
					0 => "healing",
					1 => "strength buff",
					2 => "slowdown",
					3 => "blinding",
					4 => "debuff removal",
					_ => "junk"
				};
				info!("You got a {} item.", item_got);
				game_progress.player_inventory[item] += 1;
				commands.entity(chest_tile).remove::<ChestTile>();
			}
		}
	}

}