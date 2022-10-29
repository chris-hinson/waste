use crate::player::Player;
use crate::wfc::wfc;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{Collision, collide};
use std::collections::HashMap;
use crate::world::{WorldMap, add_to_world};

pub(crate) const TILE_SIZE: f32 = 64.;
pub(crate) const MAP_WIDTH: usize = 20;
pub(crate) const MAP_HEIGHT: usize = 12;
// pub(crate) const CHUNK_WIDTH: usize  = 20   ;
// pub(crate) const CHUNK_HEIGHT: usize = 12   ;
pub(crate) const WIN_H: f32 = 768.;
pub(crate) const WIN_W: f32 = 1280.;
pub(crate) const LEVEL_WIDTH: f32 = MAP_WIDTH as f32 * TILE_SIZE;
pub(crate) const LEVEL_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_SIZE;
const DRAW_START_X: f32 = -WIN_W / 2. + TILE_SIZE / 2.;
const DRAW_START_Y: f32 = -WIN_H / 2. + TILE_SIZE / 2.;
// const DRAW_STOP_X: f32  = LEVEL_WIDTH - TILE_SIZE/2.;
// const DRAW_STOP_Y: f32  = LEVEL_HEIGHT - TILE_SIZE/2.;

pub(crate) const OVERWORLD_TILESHEET: &str = "backgrounds/overworld_tilesheet.png";

#[derive(Component)]
pub(crate) struct Tile;

#[derive(Component)]
pub(crate) struct MonsterTile {
    pub(crate) transform: Transform,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct ChunkCenter {
    pub(crate) transform: Transform,
}

impl PartialEq for ChunkCenter {
    fn eq(&self, other: &Self) -> bool {
        self.transform.translation.x == other.transform.translation.x &&
        self.transform.translation.y == other.transform.translation.y
    }
}

impl Eq for ChunkCenter {}

#[derive(Component, Debug)]
pub(crate) struct Chunk {
    pub(crate) center: ChunkCenter,
    pub(crate) tiles: Vec<Vec<usize>>,
}

pub(crate) fn init_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut world: ResMut<WorldMap>,
) {

    let starting_center = ChunkCenter {
        transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
    };
    commands.spawn().insert(starting_center);
    let starting_chunk = Chunk{
        center: starting_center,
        tiles: wfc(None),
    };

    let map_handle = asset_server.load(OVERWORLD_TILESHEET);
    let map_atlas = TextureAtlas::from_grid(map_handle, Vec2::splat(TILE_SIZE), 7, 6);

    let map_atlas_len = map_atlas.textures.len();
    let map_atlas_handle = texture_atlases.add(map_atlas.clone());

    println!("Number of texture atlases: {}", map_atlas_len);

    // from center of the screen to half a tile from edge
    // so the tile will never be "cut in half" by edge of screen
    let mut x = DRAW_START_X;
    let mut y = DRAW_START_Y;

    for i in 0..starting_chunk.tiles.len() {
        for j in 0..starting_chunk.tiles[i].len() {
            let tile = starting_chunk.tiles[i][j];
            let t = Vec3::new(x, y, -1.);
            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: map_atlas_handle.clone(),
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
                commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: map_atlas_handle.clone(),
                        transform: Transform {
                            translation: t,
                            ..default()
                        },
                        sprite: TextureAtlasSprite {
                            index: 4,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(MonsterTile {
                        transform: Transform::from_xyz(x, y, -1.),
                    });
            }

            x += TILE_SIZE;
        }
        x = DRAW_START_X;
        y += TILE_SIZE;
    }

    add_to_world(commands, world, starting_chunk, 0, 0);
}

pub(crate) fn find_next_chunk(mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut player: Query<&mut Transform, (With<Player>, Without<Tile>, Without<MonsterTile>)>,
    mut chunks: Query<&mut Chunk>,
    chunk_centers: Query<&mut ChunkCenter>,
){
    // check for collision
    if player.is_empty(){
        error!("Couldn't find player");
    }

    if chunks.is_empty(){
        error!("Couldn't find chunk");
    }

    let pt = player.single_mut();

    // Imperative that we pick a single chunk out based on some hashmap or similar rather than
    // looping over all of them each time
    for chunk in chunks.iter_mut(){
        let chunk_position = chunk.center.transform.translation;
        let collision = collide(pt.translation, Vec2::splat(320.), chunk_position, Vec2::new(WIN_W, WIN_H));
        match collision {
            None => {},
            Some(direction) => {
                match direction {
                    Collision::Top => {
                        let mut seed: Vec<(usize,(usize, usize))> = Vec::new();
                        for i in 0..MAP_WIDTH{
                            let this_tile = (chunk.tiles[MAP_HEIGHT-1][i], (0 as usize, i));
                            seed.push(this_tile);
                        }
                        let new_x = chunk_position.x;
                        let new_y = chunk_position.y + WIN_H;
                        let new_center = ChunkCenter {
                            transform: Transform::from_translation(Vec3::new(new_x, new_y, -1.)),
                        };
                        commands.spawn().insert(new_center);
                        if !chunk_centers.iter().any(|c| c == &new_center){
                            let new_chunk = Chunk{
                                center: new_center,
                                tiles: wfc(Some(seed)),
                            };
                            info!("New chunk spawned at: {:?}", new_center);
                            
                            let map_handle = asset_server.load(OVERWORLD_TILESHEET);
                            let map_atlas = TextureAtlas::from_grid(map_handle, 
                                Vec2::splat(TILE_SIZE), 7, 6);
                        
                            let map_atlas_len = map_atlas.textures.len();
                            let map_atlas_handle = texture_atlases.add(map_atlas.clone());
                        
                            println!("Number of texture atlases: {}", map_atlas_len);
                        
                            // from center of the screen to half a tile from edge
                            // so the tile will never be "cut in half" by edge of screen
                            let mut x = new_chunk.center.transform.translation.x + DRAW_START_X;
                            let mut y = new_chunk.center.transform.translation.y + DRAW_START_Y;
                            info!("new chunk: draw_start_x: {}, draw_start_y: {}", x, y);
                            // info!("tiles: {:?}", new_chunk.tiles);
                        
                            for i in 0..new_chunk.tiles.len(){
                                for j in 0..new_chunk.tiles[i].len(){
                                    let tile = new_chunk.tiles[i][j];
                                    let t = Vec3::new(x, y, -1.,);
                                    commands
                                    .spawn_bundle(SpriteSheetBundle {
                                        texture_atlas: map_atlas_handle.clone(),
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
                                        commands
                                        .spawn_bundle(SpriteSheetBundle {
                                            texture_atlas: map_atlas_handle.clone(),
                                            transform: Transform {
                                                translation: t,
                                                ..default()
                                            },
                                            sprite: TextureAtlasSprite {
                                                index: 4,
                                                ..default()
                                            },
                                            ..default()
                                        })
                                        .insert(MonsterTile{transform: Transform::from_xyz(x, y, -1.)});
                                    }
                        
                                    x += TILE_SIZE;
                                }
                                x = new_chunk.center.transform.translation.x + DRAW_START_X;
                                y += TILE_SIZE;
                            }
                            commands.spawn().insert(new_chunk);
                        
                        }
                    },
                    Collision::Bottom => {
                        let mut seed: Vec<(usize,(usize, usize))> = Vec::new();
                        for i in 0..MAP_WIDTH{
                            let this_tile = (chunk.tiles[0][i], (MAP_HEIGHT-1, i));
                            seed.push(this_tile);
                        }
                        let new_x = chunk_position.x;
                        let new_y = chunk_position.y - WIN_H;
                        let new_center = ChunkCenter {
                            transform: Transform::from_translation(Vec3::new(new_x, new_y, -1.)),
                        };
                        commands.spawn().insert(new_center);
                        if !chunk_centers.iter().any(|c| c == &new_center){
                            let new_chunk = Chunk{
                                center: new_center,
                                // Using a seed on the bottom causes an infinite loop no matter what
                                // we do, even after updating neighbor's
                                // tiles: wfc(Some(seed)),
                                tiles: wfc(None),
                            };
                            info!("New chunk spawned at: {:?}", new_center);
                            
                            let map_handle = asset_server.load(OVERWORLD_TILESHEET);
                            let map_atlas = TextureAtlas::from_grid(map_handle, 
                                Vec2::splat(TILE_SIZE), 7, 6);
                        
                            let map_atlas_len = map_atlas.textures.len();
                            let map_atlas_handle = texture_atlases.add(map_atlas.clone());
                        
                            println!("Number of texture atlases: {}", map_atlas_len);
                        
                            // from center of the screen to half a tile from edge
                            // so the tile will never be "cut in half" by edge of screen
                            let mut x = new_chunk.center.transform.translation.x + DRAW_START_X;
                            let mut y = new_chunk.center.transform.translation.y + DRAW_START_Y;
                            info!("new chunk: draw_start_x: {}, draw_start_y: {}", x, y);
                            // info!("tiles: {:?}", new_chunk.tiles);
                        
                            for i in 0..new_chunk.tiles.len(){
                                for j in 0..new_chunk.tiles[i].len(){
                                    let tile = new_chunk.tiles[i][j];
                                    let t = Vec3::new(x, y, -1.,);
                                    commands
                                    .spawn_bundle(SpriteSheetBundle {
                                        texture_atlas: map_atlas_handle.clone(),
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
                                        commands
                                        .spawn_bundle(SpriteSheetBundle {
                                            texture_atlas: map_atlas_handle.clone(),
                                            transform: Transform {
                                                translation: t,
                                                ..default()
                                            },
                                            sprite: TextureAtlasSprite {
                                                index: 4,
                                                ..default()
                                            },
                                            ..default()
                                        })
                                        .insert(MonsterTile{transform: Transform::from_xyz(x, y, -1.)});
                                    }
                        
                                    x += TILE_SIZE;
                                }
                                x = new_chunk.center.transform.translation.x + DRAW_START_X;
                                y += TILE_SIZE;
                            }
                            commands.spawn().insert(new_chunk);
                        
                        }
                    },
                    Collision::Left => {
                        let mut seed: Vec<(usize,(usize, usize))> = Vec::new();
                        for i in 0..MAP_HEIGHT{
                            let this_tile = (chunk.tiles[i][0], (i, 0 as usize));
                            seed.push(this_tile);
                        }
                        let new_x = chunk_position.x - WIN_W;
                        let new_y = chunk_position.y;
                        let new_center = ChunkCenter {
                            transform: Transform::from_translation(Vec3::new(new_x, new_y, -1.)),
                        };
                        commands.spawn().insert(new_center);
                        if !chunk_centers.iter().any(|c| c == &new_center){
                            let new_chunk = Chunk{
                                center: new_center,
                                tiles: wfc(Some(seed)),
                            };
                            info!("New chunk spawned at: {:?}", new_center);
                            
                            let map_handle = asset_server.load(OVERWORLD_TILESHEET);
                            let map_atlas = TextureAtlas::from_grid(map_handle, 
                                Vec2::splat(TILE_SIZE), 7, 6);
                        
                            let map_atlas_len = map_atlas.textures.len();
                            let map_atlas_handle = texture_atlases.add(map_atlas.clone());
                        
                            println!("Number of texture atlases: {}", map_atlas_len);
                        
                            // from center of the screen to half a tile from edge
                            // so the tile will never be "cut in half" by edge of screen
                            let mut x = new_chunk.center.transform.translation.x + DRAW_START_X;
                            let mut y = new_chunk.center.transform.translation.y + DRAW_START_Y;
                            info!("new chunk: draw_start_x: {}, draw_start_y: {}", x, y);
                            // info!("tiles: {:?}", new_chunk.tiles);
                        
                            for i in 0..new_chunk.tiles.len(){
                                for j in 0..new_chunk.tiles[i].len(){
                                    let tile = new_chunk.tiles[i][j];
                                    let t = Vec3::new(x, y, -1.,);
                                    commands
                                    .spawn_bundle(SpriteSheetBundle {
                                        texture_atlas: map_atlas_handle.clone(),
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
                                        commands
                                        .spawn_bundle(SpriteSheetBundle {
                                            texture_atlas: map_atlas_handle.clone(),
                                            transform: Transform {
                                                translation: t,
                                                ..default()
                                            },
                                            sprite: TextureAtlasSprite {
                                                index: 4,
                                                ..default()
                                            },
                                            ..default()
                                        })
                                        .insert(MonsterTile{transform: Transform::from_xyz(x, y, -1.)});
                                    }
                        
                                    x += TILE_SIZE;
                                }
                                x = new_chunk.center.transform.translation.x + DRAW_START_X;
                                y += TILE_SIZE;
                            }
                            commands.spawn().insert(new_chunk);
                        
                        }
                    },
                    Collision::Right => {
                        let mut seed: Vec<(usize,(usize, usize))> = Vec::new();
                        for i in 0..MAP_HEIGHT{
                            let this_tile = (chunk.tiles[i][MAP_WIDTH-1], (i, MAP_WIDTH-1));
                            seed.push(this_tile);
                        }
                        let new_x = chunk_position.x + WIN_W;
                        let new_y = chunk_position.y;
                        let new_center = ChunkCenter {
                            transform: Transform::from_translation(Vec3::new(new_x, new_y, -1.)),
                        };
                        commands.spawn().insert(new_center);
                        if !chunk_centers.iter().any(|c| c == &new_center){
                            let new_chunk = Chunk{
                                center: new_center,
                                tiles: wfc(Some(seed)),
                            };
                            info!("New chunk spawned at: {:?}", new_center);
                            
                            let map_handle = asset_server.load(OVERWORLD_TILESHEET);
                            let map_atlas = TextureAtlas::from_grid(map_handle, 
                                Vec2::splat(TILE_SIZE), 7, 6);
                        
                            let map_atlas_len = map_atlas.textures.len();
                            let map_atlas_handle = texture_atlases.add(map_atlas.clone());
                        
                            // from center of the screen to half a tile from edge
                            // so the tile will never be "cut in half" by edge of screen
                            let mut x = new_chunk.center.transform.translation.x + DRAW_START_X;
                            let mut y = new_chunk.center.transform.translation.y + DRAW_START_Y;
                            // info!("tiles: {:?}", new_chunk.tiles);
                        
                            for i in 0..new_chunk.tiles.len(){
                                for j in 0..new_chunk.tiles[i].len(){
                                    let tile = new_chunk.tiles[i][j];
                                    let t = Vec3::new(x, y, -1.,);
                                    commands
                                    .spawn_bundle(SpriteSheetBundle {
                                        texture_atlas: map_atlas_handle.clone(),
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
                                        commands
                                        .spawn_bundle(SpriteSheetBundle {
                                            texture_atlas: map_atlas_handle.clone(),
                                            transform: Transform {
                                                translation: t,
                                                ..default()
                                            },
                                            sprite: TextureAtlasSprite {
                                                index: 4,
                                                ..default()
                                            },
                                            ..default()
                                        })
                                        .insert(MonsterTile{transform: Transform::from_xyz(x, y, -1.)});
                                    }
                        
                                    x += TILE_SIZE;
                                }
                                x = new_chunk.center.transform.translation.x + DRAW_START_X;
                                y += TILE_SIZE;
                            }
                            commands.spawn().insert(new_chunk);
                        
                        }

                    },
                    Collision::Inside => {},
                }
            }
        }
    }

}