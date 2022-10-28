use {bevy::prelude::*};
use crate::Chunk;
use std::collections::HashMap;


#[derive(Default)]
pub(crate) struct WorldMap{
    // the first usize is the chunk id
    // the tuple is chunks relative(logical) position
    pub(crate) positions: HashMap<usize, (isize, isize)>,
    // backward lookup that everyone loves
    // maybe this is useful maybe its not I dont care
    pub(crate) chunk_ids: HashMap<(isize, isize), usize>,
    // AFAIK, there is no way to get an Entity from an EntityId
    // which is INSANE
    // so here it is:
    pub(crate) chunks: HashMap<usize, Entity>,
}


pub(crate) fn add_to_world(mut commands: Commands, mut world: ResMut<WorldMap>, chunk: Chunk, x: isize, y: isize){
    let entity = commands.spawn().insert(chunk).id();
    let id = entity.id();
    world.positions.insert(id as usize, (x, y));
    world.chunk_ids.insert((x, y), id as usize);
    world.chunks.insert(id as usize, entity);
}

pub(crate) fn get_east(mut world: ResMut<WorldMap>, x: isize, y: isize) -> Option<Entity>{
    let id: Option<&usize> = world.chunk_ids.get(&(x+1, y));
    if let Some(id) = id {
        let entity: Option<&Entity> = world.chunks.get(id);
        if let Some(entity) = entity {
            Some(*entity)
        }else{
            None
        }
    }else{
        None
    }
}

pub(crate) fn get_west(mut world: ResMut<WorldMap>, x: isize, y: isize) -> Option<Entity>{
    let id: Option<&usize> = world.chunk_ids.get(&(x-1, y));
    if let Some(id) = id {
        let entity: Option<&Entity> = world.chunks.get(id);
        if let Some(entity) = entity {
            Some(*entity)
        }else{
            None
        }
    }else{
        None
    }
}

pub(crate) fn get_north(mut world: ResMut<WorldMap>, x: isize, y: isize) -> Option<Entity>{
    let id: Option<&usize> = world.chunk_ids.get(&(x, y+1));
    if let Some(id) = id {
        let entity: Option<&Entity> = world.chunks.get(id);
        if let Some(entity) = entity {
            Some(*entity)
        }else{
            None
        }
    }else{
        None
    }
}

pub(crate) fn get_south(mut world: ResMut<WorldMap>, x: isize, y: isize) -> Option<Entity>{
    let id: Option<&usize> = world.chunk_ids.get(&(x, y-1));
    if let Some(id) = id {
        let entity: Option<&Entity> = world.chunks.get(id);
        if let Some(entity) = entity {
            Some(*entity)
        }else{
            None
        }
    }else{
        None
    }
}

pub(crate) fn logical_to_rendering(x: isize, y: isize) -> (f32, f32){
    (x as f32 * 1280.0, y as f32 * 768.0)
}