use {bevy::prelude::*};
use bevy::ecs::entity;

use crate::{Chunk, backgrounds::{WIN_W, WIN_H}, monster::MonsterBundle};
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

    pub(crate) fn get_east(&self, x: isize, y: isize) -> Option<Chunk>{
        self.get_chunk(x + 1, y)
    }
    
    pub(crate) fn get_west(&mut self, x: isize, y: isize) -> Option<Chunk>{
        self.get_chunk(x - 1, y)
    }
    
    pub(crate) fn get_north(&mut self, x: isize, y: isize) -> Option<Chunk>{
        self.get_chunk(x, y + 1)
    }
    
    pub(crate) fn get_south(&mut self, x: isize, y: isize) -> Option<Chunk>{
        self.get_chunk(x, y - 1)
    }

}
    
pub(crate) fn logical_to_rendering(x: isize, y: isize) -> (f32, f32){
    (x as f32 * WIN_W, y as f32 * WIN_H)
}

#[derive(Default)]
pub(crate) struct GameProgress{
    // the level of our player, which is also the level we should spawn the monsters
    pub current_level: usize,
    // the number of monsters we have defeated before next level
    pub level_progress: usize,
    // number of monsters defeated before the boss
    pub boss_progress: usize,
    // level_progress needed to see the boss
    pub boss_level: usize,
    // number of monsters defeated before the next level
    pub next_level: usize,
    // keeps track of how many monsters we have
    // this is the our id independent from bevy's entity id
    pub num_monsters: usize,
    // keeps track of all monsters we currently have
    // HashMap<our_id, bevy's entity id>
    pub allied_monster_id: HashMap<usize, usize>,
    // back look up table
    pub id_allied_monster: HashMap<usize, usize>,
    // our id to Entity
    pub monster_id_entity: HashMap<usize, Entity>,
    // Entity to our id
    pub entity_monster_id: HashMap<Entity, usize>,
    // get MonsterBundle from entity
    pub monster_entity_id_to_stats: HashMap<usize, MonsterBundle>,
}


impl GameProgress {
    pub fn new_monster(&mut self, entity: Entity, stats: MonsterBundle) {
        let id = entity.id() as usize;
        self.num_monsters += 1;
        self.allied_monster_id.insert(self.num_monsters, id);
        self.id_allied_monster.insert(id, self.num_monsters);
        self.monster_id_entity.insert(self.num_monsters, entity);
        self.entity_monster_id.insert(entity, self.num_monsters);
        self.monster_entity_id_to_stats.insert(id, stats);
    }

    pub fn next_monster(&mut self, last_monster: Entity) -> Option<&Entity> {
        let our_id = self.entity_monster_id.get(&last_monster);
        let next_montser = self.monster_id_entity.get(&(*our_id.unwrap()+1));
        return next_montser;
    }
}

// We can reintroduce this once we want/need a fancy resource
// impl FromWorld for GameProgress {
//     fn from_world(world: &mut World) -> Self {
//         // I dont know yet, this is a big thing to do
//         return GameProgress{
//             current_level: 0,
//             level_progress: 0,
//             boss_progress: 0,
//             boss_level: 0,
//             next_level: 0,
//             num_monsters: 0,
//             allied_monster_id: HashMap::new(),
//             id_allied_monster: HashMap::new(),
//             monster_entity_id_to_stats: HashMap::new(),
//         }
//     }
// }