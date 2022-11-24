use {bevy::prelude::*};
use bevy::ecs::entity;

use crate::{Chunk, backgrounds::{WIN_W, WIN_H}, monster::{MonsterStats, Element}};
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

pub(crate) struct GameProgress{
    /// the level of our player, which is also the level we should spawn the monsters
    pub(crate) current_level: usize,
    /// number of bosses we have defeated
    pub(crate) num_boss_defeated: usize,
    /// if we have defeated the level boss
    pub(crate) level_boss_awaken: bool,
    /// keeps track of how many monsters we have
    /// this is the our id independent from bevy's entity id
    pub(crate) num_monsters: usize,
    /// Number of monsters currently available in the party
    /// 
    /// Initialized to the same amount as num_monsters and as monsters die it is decremented
    pub(crate) num_living_monsters: usize,
    /// keeps track of all monsters we currently have
    /// HashMap<our_id, bevy's entity id>
    pub(crate) allied_monster_id: HashMap<usize, usize>,
    /// back look up table
    pub(crate) id_allied_monster: HashMap<usize, usize>,
    /// our id to Entity
    pub(crate) monster_id_entity: HashMap<usize, Entity>,
    /// Entity to our id
    pub(crate) entity_monster_id: HashMap<Entity, usize>,
    /// get MonsterBundle from entity
    pub(crate) monster_entity_to_stats: HashMap<Entity, MonsterStats>,
    /// all monsters' entity id to their stats
    /// to help us retrieve stats when we defeat them
    pub(crate) enemy_stats: HashMap<Entity, MonsterStats>,
    /// Number of items left of each type
    /// Heal Item = 0, Strength Item = 1, Slow Item = 2, Blinding Item = 3
    /// Debuff Removal Item = 4
    pub(crate) player_inventory: Vec<usize>,
    /// Number of turns remaining with a given buff applied
    /// Strength Buff = 0, Slowness = 1, Blindness = 2
    pub(crate) turns_left_of_buff: Vec<usize>,
}


impl GameProgress {
    pub fn new_monster(&mut self, entity: Entity, stats: MonsterStats) {
        let id = entity.id() as usize;
        self.allied_monster_id.insert(self.num_monsters, id);
        self.id_allied_monster.insert(id, self.num_monsters);
        self.monster_id_entity.insert(self.num_monsters, entity);
        self.entity_monster_id.insert(entity, self.num_monsters);
        self.monster_entity_to_stats.insert(entity, stats);
        self.num_monsters += 1;
        self.num_living_monsters += 1;
        info!("you have {} monsters now.", self.num_monsters);
    }

    pub fn next_monster(&mut self, last_monster: Entity) -> Option<&Entity> {
        let our_id = self.entity_monster_id.get(&last_monster);
        let next_montser = self.monster_id_entity.get(&(*our_id.unwrap()+1));
        return next_montser;
    }

    /// Cycle through available monsters
    pub fn next_monster_cyclic(&mut self, last_monster: Entity) -> Option<&Entity> {
        if self.num_living_monsters <= 0 {
            return None;
        }
        let monster_id_entity_len = self.monster_id_entity.len();
        let our_id = self.entity_monster_id.get(&last_monster);
        info!("Trying {} with length {}", ((*our_id.unwrap()+1) % monster_id_entity_len), monster_id_entity_len);
        let next_montser = self.monster_id_entity.get(&(((*our_id.unwrap()+1) % monster_id_entity_len)));
        next_montser
    }

    pub fn win_battle(&mut self){
        self.num_living_monsters = self.num_monsters;
        self.current_level += 1;
        if self.current_level % 5 == 0 {
            // We hit a level appropriate to fight a boss
            self.level_boss_awaken = true;
            info!("You are now level {}. You have awakened a boss! Your next fight will be a boss fight...",
                self.current_level);
        } else {
            info!("You are now level {}. Defeat {} more monsters to face the next boss!",
                self.current_level,
                5-(self.current_level%5));
        }
    }

    pub fn win_boss(&mut self){
        self.num_living_monsters = self.num_monsters;
        self.current_level += 1;
        self.num_boss_defeated += 1;
        self.level_boss_awaken = false;
        info!("You have defeated {} bosses.", self.num_boss_defeated);
        if self.num_boss_defeated == 5{
            info!("You have defeated all the bosses, you win!");
            // win the game
            // commands.insert_resource(NextState(GameState::Credits));
        }
    }
}

impl Default for GameProgress {
    fn default() -> Self {
        Self { 
            current_level: 1 as usize, 
            num_boss_defeated: Default::default(),
            level_boss_awaken: Default::default(), 
            num_monsters: Default::default(), 
            num_living_monsters: Default::default(),
            allied_monster_id: Default::default(), 
            id_allied_monster: Default::default(), 
            monster_id_entity: Default::default(), 
            entity_monster_id: Default::default(), 
            monster_entity_to_stats: Default::default(), 
            enemy_stats: Default::default(),
            player_inventory: vec![0; 9],
            turns_left_of_buff: vec![0; 3]
        }
    }
}


#[derive(Clone, Copy)]
pub(crate) struct TypeSystem {
    pub type_modifier: [[f32; 8]; 8],
}

impl Default for TypeSystem{
    fn default() -> Self {
        let mut modifier_map: [[f32; 8]; 8] = [[1.0; 8]; 8];
        // Scav = 0
        modifier_map[Element::Scav as usize][Element::Scav as usize] = 1.0;
        modifier_map[Element::Scav as usize][Element::Growth as usize] = 2.0;
        modifier_map[Element::Scav as usize][Element::Ember as usize] = 0.5;
        modifier_map[Element::Scav as usize][Element::Flood as usize] = 2.0;
        modifier_map[Element::Scav as usize][Element::Rad as usize] = 0.5;
        modifier_map[Element::Scav as usize][Element::Robot as usize] = 1.0;
        modifier_map[Element::Scav as usize][Element::Clean as usize] = 0.5;
        modifier_map[Element::Scav as usize][Element::Filth as usize] = 0.5;
        // Growth = 1
        modifier_map[Element::Growth as usize][Element::Scav as usize] = 1.0;
        modifier_map[Element::Growth as usize][Element::Growth as usize] = 1.0;
        modifier_map[Element::Growth as usize][Element::Ember as usize] = 0.5;
        modifier_map[Element::Growth as usize][Element::Flood as usize] = 2.0;
        modifier_map[Element::Growth as usize][Element::Rad as usize] = 2.0;
        modifier_map[Element::Growth as usize][Element::Robot as usize] = 2.0;
        modifier_map[Element::Growth as usize][Element::Clean as usize] = 0.5;
        modifier_map[Element::Growth as usize][Element::Filth as usize] = 0.5;
        // Ember = 2
        modifier_map[Element::Ember as usize][Element::Scav as usize] = 2.0;
        modifier_map[Element::Ember as usize][Element::Growth as usize] = 2.0;
        modifier_map[Element::Ember as usize][Element::Ember as usize] = 1.0;
        modifier_map[Element::Ember as usize][Element::Flood as usize] = 0.5;
        modifier_map[Element::Ember as usize][Element::Rad as usize] = 2.0;
        modifier_map[Element::Ember as usize][Element::Robot as usize] = 0.5;
        modifier_map[Element::Ember as usize][Element::Clean as usize] = 1.0;
        modifier_map[Element::Ember as usize][Element::Filth as usize] = 0.5;
        // Flood = 3
        modifier_map[Element::Flood as usize][Element::Scav as usize] = 1.0;
        modifier_map[Element::Flood as usize][Element::Growth as usize] = 0.5;
        modifier_map[Element::Flood as usize][Element::Ember as usize] = 2.0;
        modifier_map[Element::Flood as usize][Element::Flood as usize] = 1.0;
        modifier_map[Element::Flood as usize][Element::Rad as usize] = 0.5;
        modifier_map[Element::Flood as usize][Element::Robot as usize] = 2.0;
        modifier_map[Element::Flood as usize][Element::Clean as usize] = 2.0;
        modifier_map[Element::Flood as usize][Element::Filth as usize] = 2.0;
        // Rad = 4
        modifier_map[Element::Rad as usize][Element::Scav as usize] = 2.0;
        modifier_map[Element::Rad as usize][Element::Growth as usize] = 0.5;
        modifier_map[Element::Rad as usize][Element::Ember as usize] = 1.0;
        modifier_map[Element::Rad as usize][Element::Flood as usize] = 1.0;
        modifier_map[Element::Rad as usize][Element::Rad as usize] = 0.5;
        modifier_map[Element::Rad as usize][Element::Robot as usize] = 0.5;
        modifier_map[Element::Rad as usize][Element::Clean as usize] = 2.0;
        modifier_map[Element::Rad as usize][Element::Filth as usize] = 1.0;
        // Robot = 5
        modifier_map[Element::Robot as usize][Element::Scav as usize] = 2.0;
        modifier_map[Element::Robot as usize][Element::Growth as usize] = 0.5;
        modifier_map[Element::Robot as usize][Element::Ember as usize] = 1.0;
        modifier_map[Element::Robot as usize][Element::Flood as usize] = 0.5;
        modifier_map[Element::Robot as usize][Element::Rad as usize] = 2.0;
        modifier_map[Element::Robot as usize][Element::Robot as usize] = 1.0;
        modifier_map[Element::Robot as usize][Element::Clean as usize] = 0.5;
        modifier_map[Element::Robot as usize][Element::Filth as usize] = 2.0;
        // Clean = 6
        modifier_map[Element::Clean as usize][Element::Scav as usize] = 0.5;
        modifier_map[Element::Clean as usize][Element::Growth as usize] = 1.0;
        modifier_map[Element::Clean as usize][Element::Ember as usize] = 2.0;
        modifier_map[Element::Clean as usize][Element::Flood as usize] = 0.5;
        modifier_map[Element::Clean as usize][Element::Rad as usize] = 1.0;
        modifier_map[Element::Clean as usize][Element::Robot as usize] = 0.5;
        modifier_map[Element::Clean as usize][Element::Clean as usize] = 1.0;
        modifier_map[Element::Clean as usize][Element::Filth as usize] = 2.0;
        // Filth = 7
        modifier_map[Element::Filth as usize][Element::Scav as usize] = 2.0;
        modifier_map[Element::Filth as usize][Element::Growth as usize] = 1.0;
        modifier_map[Element::Filth as usize][Element::Ember as usize] = 0.5;
        modifier_map[Element::Filth as usize][Element::Flood as usize] = 1.0;
        modifier_map[Element::Filth as usize][Element::Rad as usize] = 1.0;
        modifier_map[Element::Filth as usize][Element::Robot as usize] = 2.0;
        modifier_map[Element::Filth as usize][Element::Clean as usize] = 2.0;
        modifier_map[Element::Filth as usize][Element::Filth as usize] = 0.5;
        return TypeSystem { type_modifier: modifier_map};
    }
}