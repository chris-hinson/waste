use {bevy::prelude::*};
use bevy::ecs::entity;

use crate::Chunk;
use std::collections::HashMap;


#[derive(Default, Debug)]
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


impl WorldMap{
    pub(crate) fn add_to_world(&mut self, entity: Entity, x: isize, y: isize){
        let id = entity.id();
        self.positions.insert(id as usize, (x, y));
        self.chunk_ids.insert((x, y), id as usize);
        self.chunks.insert(id as usize, entity);
    }

    pub(crate) fn get_east(&mut self, x: isize, y: isize) -> Option<Entity>{
        let id: Option<&usize> = self.chunk_ids.get(&(x+1, y));
        if let Some(id) = id {
            let entity: Option<&Entity> = self.chunks.get(id);
            if let Some(entity) = entity {
                Some(*entity)
            }else{
                None
            }
        }else{
            None
        }
    }
    
    pub(crate) fn get_west(&mut self, x: isize, y: isize) -> Option<Entity>{
        let id: Option<&usize> = self.chunk_ids.get(&(x-1, y));
        if let Some(id) = id {
            let entity: Option<&Entity> = self.chunks.get(id);
            if let Some(entity) = entity {
                Some(*entity)
            }else{
                None
            }
        }else{
            None
        }
    }
    
    pub(crate) fn get_north(&mut self, x: isize, y: isize) -> Option<Entity>{
        let id: Option<&usize> = self.chunk_ids.get(&(x, y+1));
        if let Some(id) = id {
            let entity: Option<&Entity> = self.chunks.get(id);
            if let Some(entity) = entity {
                Some(*entity)
            }else{
                None
            }
        }else{
            None
        }
    }
    
    pub(crate) fn get_south(&mut self, x: isize, y: isize) -> Option<Entity>{
        let id: Option<&usize> = self.chunk_ids.get(&(x, y-1));
        if let Some(id) = id {
            let entity: Option<&Entity> = self.chunks.get(id);
            if let Some(entity) = entity {
                Some(*entity)
            }else{
                None
            }
        }else{
            None
        }
    }

}
    




pub(crate) fn logical_to_rendering(x: isize, y: isize) -> (f32, f32){
    (x as f32 * 1280.0, y as f32 * 768.0)
}