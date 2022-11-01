use {bevy::prelude::*};
use bevy::ecs::entity;

use crate::{Chunk, backgrounds::{WIN_W, WIN_H}};
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
    // AFAIK, there is no way to get a component from an Entity
    // which is GREAT
    // so here it is:
    pub(crate) chunk_components: HashMap<usize, Chunk>,
}


impl WorldMap{
    pub(crate) fn add_to_world(&mut self, chunk: Chunk, entity: Entity, x: isize, y: isize){
        let id = entity.id();
        self.positions.insert(id as usize, (x, y));
        self.chunk_ids.insert((x, y), id as usize);
        self.chunks.insert(id as usize, entity);
        self.chunk_components.insert(id as usize, chunk);
    }

    pub(crate) fn get_chunk(&self, x: isize, y: isize) -> Option<Chunk>{
        let id = self.chunk_ids.get(&(x, y));
        if let Some(id) = id{
            let chunk = self.chunk_components.get(id);
            if let Some(chunk) = chunk{
                Some(chunk.clone())
            }else {
                None
            }
        }else {
            None
        }
        
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
    (x as f32 * WIN_W, y as f32 * WIN_H)
}