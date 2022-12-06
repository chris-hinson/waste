use crate::player::Player;
use crate::quests::{Quest, NPC, NPC_PATH};
use crate::wfc::{wfc, ProcGen};
use crate::world::{logical_to_rendering, WorldMap};
use bevy::prelude::*;

pub(crate) const TILE_SIZE: f32 = 64.;
pub(crate) const MAP_WIDTH: usize = 20;
pub(crate) const MAP_HEIGHT: usize = 12;
pub(crate) const WIN_H: f32 = 768.;
pub(crate) const WIN_W: f32 = 1280.;
const DRAW_START_X: f32 = -WIN_W / 2. + TILE_SIZE / 2.;
const DRAW_START_Y: f32 = WIN_H / 2. - TILE_SIZE / 2.;
// pub(crate) const LEVEL_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;
// pub(crate) const LEVEL_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE;

pub(crate) const OVERWORLD_TILESHEET: &str = "backgrounds/overworld_tilesheet.png";

#[derive(Component)]
pub(crate) struct Tile;

/// Tile to start a battle
#[derive(Component)]
pub(crate) struct MonsterTile;

/// Tile to automatically heal all monsters
#[derive(Component)]
pub(crate) struct HealingTile;

/// Tile to represent an item-giving chest
#[derive(Component)]
pub(crate) struct ChestTile;

#[derive(Component, Debug, Clone)]
pub(crate) struct Chunk {
    pub(crate) position: (isize, isize),
    pub(crate) tiles: Vec<Vec<usize>>,
}

macro_rules! draw_chunk {
    ($chunk:expr, $commands:expr, $map_atlas_handle:expr) => {
        // from center of the screen to half a tile from edge
        // so the tile will never be "cut in half" by edge of screen
        let rendering_center = logical_to_rendering($chunk.position.0, $chunk.position.1);
        // info!("Rendering chunk at {:?}", rendering_center);
        let mut x = rendering_center.0 + DRAW_START_X;
        let mut y = rendering_center.1 + DRAW_START_Y;

        for i in 0..$chunk.tiles.len() {
            for j in 0..$chunk.tiles[i].len() {
                let tile = $chunk.tiles[i][j];
                let t = Vec3::new(x, y, -1.);
                $commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: $map_atlas_handle.clone(),
                        transform: Transform {
                            translation: t,
                            ..default()
                        },
                        sprite: TextureAtlasSprite {
                            index: tile,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(Tile);
                if tile == 4 {
                    $commands
                        .spawn_bundle(SpriteSheetBundle {
                            texture_atlas: $map_atlas_handle.clone(),
                            transform: Transform {
                                translation: t,
                                ..default()
                            },
                            sprite: TextureAtlasSprite {
                                index: tile,
                                ..default()
                            },
                            ..default()
                        })
                        .insert(MonsterTile);
                }
                if tile == 31 {
                    $commands
                        .spawn_bundle(SpriteSheetBundle {
                            texture_atlas: $map_atlas_handle.clone(),
                            transform: Transform {
                                translation: t,
                                ..default()
                            },
                            sprite: TextureAtlasSprite {
                                index: tile,
                                ..default()
                            },
                            ..default()
                        })
                        .insert(HealingTile);
                }
                if tile == 33 {
                    $commands
                        .spawn_bundle(SpriteSheetBundle {
                            texture_atlas: $map_atlas_handle.clone(),
                            transform: Transform {
                                translation: t,
                                ..default()
                            },
                            sprite: TextureAtlasSprite {
                                index: tile,
                                ..default()
                            },
                            ..default()
                        })
                        .insert(ChestTile);
                }

                x += TILE_SIZE;
            }
            x = rendering_center.0 + DRAW_START_X;
            y -= TILE_SIZE;
        }
    };
}

pub(crate) fn init_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut world: ResMut<WorldMap>,
    procgen: Res<ProcGen>,
) {
    let rules = procgen.rules.clone();

    let starting_chunk = Chunk {
        position: (0, 0),
        tiles: wfc(None, rules),
    };

    let map_handle = asset_server.load(OVERWORLD_TILESHEET);
    let map_atlas = TextureAtlas::from_grid(map_handle, Vec2::splat(TILE_SIZE), 7, 6);

    let map_atlas_handle = texture_atlases.add(map_atlas);

    let entity = commands.spawn().insert(starting_chunk.clone()).id();
    world.add_to_world(starting_chunk.clone(), entity, 0, 0);

    draw_chunk!(starting_chunk, commands, map_atlas_handle);

    // Spawn a few NPCs at fixed locations
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(NPC_PATH),
            transform: Transform::from_xyz(1000., 1000., 0.),
            ..default()
        })
        .insert(NPC {
            quest: Quest::random(),
        });

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(NPC_PATH),
            transform: Transform::from_xyz(-200., 200., 0.),
            ..default()
        })
        .insert(NPC {
            quest: Quest::random(),
        });

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(NPC_PATH),
            transform: Transform::from_xyz(-5000., -5000., 0.),
            ..default()
        })
        .insert(NPC {
            quest: Quest::random(),
        });
}

pub(crate) fn expand_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut world: ResMut<WorldMap>,
    procgen: Res<ProcGen>,
    mut player_query: Query<&mut Player>,
) {
    // check for collision
    if player_query.is_empty() {
        error!("Couldn't find player");
    }

    let rules = procgen.rules.clone();

    let player = player_query.single_mut();
    let player_chunk_pos = player.current_chunk;
    // These unwraps could panic if the player goes off the map?
    // Get the chunk the player is in
    let pc_x = player_chunk_pos.0;
    let pc_y = player_chunk_pos.1;
    let player_chunk = world.get_chunk(pc_x, pc_y).unwrap();

    let map_handle = asset_server.load(OVERWORLD_TILESHEET);
    let map_atlas = TextureAtlas::from_grid(map_handle, Vec2::splat(TILE_SIZE), 7, 6);

    let map_atlas_handle = texture_atlases.add(map_atlas);

    let east_chunk = world.chunk_ids.get(&(pc_x + 1, pc_y));
    match east_chunk {
        Some(_) => (),
        None => {
            // Prevent the system from trying to generate a chunk here again
            let (x, y) = (player_chunk_pos.0 + 1, player_chunk_pos.1);
            world.chunk_ids.insert((x, y), 0);
            // Generate seeding vector
            let mut seed: Vec<(usize, (usize, usize))> = Vec::new();
            for i in 0..MAP_HEIGHT {
                seed.push((player_chunk.tiles[i][MAP_WIDTH - 1], (i, 0)));
            }
            // if world.get_chunk(x+1, y).is_some(){
            //     for i in 0..MAP_HEIGHT{
            //         seed.push((world.get_chunk(x+1, y).unwrap().tiles[i][0], (i, MAP_WIDTH - 1)));
            //     }
            // }
            // if world.get_chunk(x, y+1).is_some(){
            //     for i in 0..MAP_WIDTH{
            //         seed.push((world.get_chunk(x, y+1).unwrap().tiles[0][i], (MAP_HEIGHT - 1, i)));
            //     }
            // }
            // if world.get_chunk(x, y-1).is_some(){
            //     for i in 0..MAP_WIDTH{
            //         seed.push((world.get_chunk(x, y-1).unwrap().tiles[MAP_HEIGHT - 1][i], (0, i)));
            //     }
            // }
            let new_chunk = Chunk {
                position: (x, y),
                tiles: wfc(Some(seed), rules.clone()),
                // tiles: wfc(None),
            };
            // info!("New chunk generated at {:?}", new_chunk.position);
            // info!("World: {:?}", world.chunk_ids);
            let entity = commands.spawn().insert(new_chunk.clone()).id();
            // Add to world before drawing, so there is no chance it being redrawn because it's not in the world
            world.add_to_world(new_chunk.clone(), entity, x, y);
            draw_chunk!(new_chunk, commands, map_atlas_handle);
        }
    }

    let south_chunk = world
        .chunk_ids
        .get(&(player_chunk_pos.0, player_chunk_pos.1 + 1));
    match south_chunk {
        Some(_) => (),
        None => {
            // Prevent the system from trying to generate another chunk here while we're drawing this one
            let (x, y) = (player_chunk_pos.0, player_chunk_pos.1 + 1);
            world.chunk_ids.insert((x, y), 0);
            // Generate seeding vector
            let mut seed: Vec<(usize, (usize, usize))> = Vec::new();
            for i in 0..MAP_WIDTH {
                seed.push((player_chunk.tiles[0][i], (MAP_HEIGHT - 1, i)));
            }
            // if world.get_chunk(x, y+1).is_some(){
            //     for i in 0..MAP_WIDTH{
            //         seed.push((world.get_chunk(x, y+1).unwrap().tiles[0][i], (0, i)));
            //     }
            // }
            // if world.get_chunk(x+1, y).is_some(){
            //     for i in 0..MAP_HEIGHT{
            //         seed.push((world.get_chunk(x+1, y).unwrap().tiles[i][0], (i, MAP_WIDTH - 1)));
            //     }
            // }
            // if world.get_chunk(x-1, y).is_some(){
            //     for i in 0..MAP_HEIGHT{
            //         seed.push((world.get_chunk(x-1, y).unwrap().tiles[i][MAP_WIDTH - 1], (i, 0)));
            //     }
            // }
            let new_chunk = Chunk {
                position: (x, y),
                tiles: wfc(Some(seed), rules.clone()),
                // tiles: wfc(None),
            };
            // info!("New chunk generated at {:?}", new_chunk.position);
            // info!("World: {:?}", world.chunk_ids);
            // draw_chunk!(new_chunk, commands, map_atlas_handle);
            let entity = commands.spawn().insert(new_chunk.clone()).id();
            world.add_to_world(new_chunk.clone(), entity, x, y);
            draw_chunk!(new_chunk, commands, map_atlas_handle);
        }
    }

    let west_chunk = world
        .chunk_ids
        .get(&(player_chunk_pos.0 - 1, player_chunk_pos.1));
    match west_chunk {
        Some(_) => (),
        None => {
            // Prevent the system from trying to generate another chunk here while we're drawing this one
            let (x, y) = (player_chunk_pos.0 - 1, player_chunk_pos.1);
            world.chunk_ids.insert((x, y), 0);
            // Generate seeding vector
            let mut seed: Vec<(usize, (usize, usize))> = Vec::new();
            for i in 0..MAP_HEIGHT {
                seed.push((player_chunk.tiles[i][0], (i, MAP_WIDTH - 1)));
            }
            // if world.get_chunk(x-1, y).is_some(){
            //     for i in 0..MAP_HEIGHT{
            //         seed.push((world.get_chunk(x-1, y).unwrap().tiles[i][MAP_WIDTH-1], (i, 0)));
            //     }
            // }
            // if world.get_chunk(x, y+1).is_some(){
            //     for i in 0..MAP_WIDTH{
            //         seed.push((world.get_chunk(x, y+1).unwrap().tiles[0][i], (MAP_HEIGHT-1, i)));
            //     }
            // }
            // if world.get_chunk(x, y-1).is_some(){
            //     for i in 0..MAP_WIDTH{
            //         seed.push((world.get_chunk(x, y-1).unwrap().tiles[MAP_HEIGHT-1][i], (0, i)));
            //     }
            // }
            let new_chunk = Chunk {
                position: (x, y),
                tiles: wfc(Some(seed), rules.clone()),
                // tiles: wfc(None),
            };
            // info!("New chunk generated at {:?}", new_chunk.position);
            // info!("World: {:?}", world.chunk_ids);
            // draw_chunk!(new_chunk, commands, map_atlas_handle);
            let entity = commands.spawn().insert(new_chunk.clone()).id();
            world.add_to_world(new_chunk.clone(), entity, x, y);
            draw_chunk!(new_chunk, commands, map_atlas_handle);
        }
    }

    let north_chunk = world
        .chunk_ids
        .get(&(player_chunk_pos.0, player_chunk_pos.1 - 1));
    match north_chunk {
        Some(_) => {}
        None => {
            // Prevent the system from trying to generate another chunk here while we're drawing this one
            let (x, y) = (player_chunk_pos.0, player_chunk_pos.1 - 1);
            world.chunk_ids.insert((x, y), 0);
            // Generate seeding vector
            let mut seed: Vec<(usize, (usize, usize))> = Vec::new();
            for i in 0..MAP_WIDTH {
                seed.push((player_chunk.tiles[MAP_HEIGHT - 1][i], (0, i)));
            }
            // if world.get_chunk(x, y-1).is_some(){
            //     for i in 0..MAP_WIDTH{
            //         seed.push((world.get_chunk(x, y-1).unwrap().tiles[MAP_HEIGHT-1][i], (MAP_HEIGHT-1, i)));
            //     }
            // }
            // if world.get_chunk(x+1, y).is_some(){
            //     for i in 0..MAP_HEIGHT{
            //         seed.push((world.get_chunk(x+1, y).unwrap().tiles[i][0], (i, MAP_WIDTH-1)));
            //     }
            // }
            // if world.get_chunk(x-1, y).is_some(){
            //     for i in 0..MAP_HEIGHT{
            //         seed.push((world.get_chunk(x-1, y).unwrap().tiles[i][MAP_WIDTH-1], (i, 0)));
            //     }
            // }
            let new_chunk = Chunk {
                position: (x, y),
                tiles: wfc(Some(seed), rules),
                // tiles: wfc(None),
            };
            // info!("New chunk generated at {:?}", new_chunk.position);
            // info!("World: {:?}", world.chunk_ids);
            // draw_chunk!(new_chunk, commands, map_atlas_handle);
            let entity = commands.spawn().insert(new_chunk.clone()).id();
            world.add_to_world(new_chunk.clone(), entity, x, y);
            draw_chunk!(new_chunk, commands, map_atlas_handle);
        }
    }
}

// macro_rules! set_seed_ {
//     ($x: expr, $y:expr, $dir:expr) => {
//         let mut seed: Vec<(usize, (usize, usize))> = Vec::new();
//         let player_north = world.get_north(x, y);
//         let player_south = world.get_south(x, y);
//         let player_east = world.get_east(x, y);
//         let player_west = world.get_west(x, y);
//         if dir = 0{ // when we go to the east
//             seed.push((chunk.tiles[i][MAP_WIDTH - 1], (i, MAP_WIDTH - 1)))
//             match player_north {
//                 Some(chunk) => {
//                     for i in 0..MAP_WIDTH{
//                         seed.push((chunk.tiles[MAP_HEIGHT - 1][i], (MAP_HEIGHT - 1, i)));
//                     }
//                 },
//                 None => (),
//             }

//         }
//     };
// }
