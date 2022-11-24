// current implementation focuses on getting all essential data for monsters in game, will need further optimizations in here and functions later.

use std::f32::consts::E;

use bevy::{prelude::*};
use iyes_loopless::prelude::*;
use crate::GameState;
use crate::camera::{SlidesCamera};
use crate::player::{Player};
use crate::backgrounds::{Tile};
use rand::{
    seq::SliceRandom,
    distributions::{Distribution, Standard},
};

pub(crate) struct MonsterPlugin;

// Elemental types
#[derive(Component, Copy, Clone, Debug)]
pub(crate) enum Element {
    Scav,
    Growth,
    Ember,
    Flood,
    Rad,
    Robot,
    Clean,
    Filth,
}

// stats, Components used for MonsterBundle
#[derive(Component, Copy, Clone)]
pub(crate) struct Level{
	pub level: usize,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct Experience(u16);
#[derive(Component, Copy, Clone)]
pub(crate) struct Health {
	pub max_health: usize,
	pub health: isize,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct Vitality(u8);
#[derive(Component, Copy, Clone)]
pub(crate) struct Strength {
    pub atk: usize,
    pub crt: usize,
    pub crt_dmg: usize,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct Defense {
    pub def: usize,
    pub crt_res: usize,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct Speed {
    pub spd: usize,
}
#[derive(Component, Copy, Clone)]
// keeps track of the number of Actions per Turn a monster has (1-3 for now) (4 for bosses)
// What? Doesn't this violate the basis of a turn-based game?
pub(crate) struct Actions(u8);
// to keep track of Known Moves a monster has (1-4, has to know 1)
// also used to keep track of the move in a turn
#[derive(Component, Copy, Clone)]
pub(crate) struct Moves {
    // known is the number of moves a monster knows
    pub known: usize,
    pub chosen: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Move {
    Attack,
    Defend,
    Heal,
    Buff,
}
// keeps track of which slot in the party a monster is in. (0 by default means not in the party)
#[derive(Component, Copy, Clone)]
pub(crate) struct Slot(u8);

// tells you if a monster is an enemy or friend (in-party true)
#[derive(Component, Copy, Clone)]
pub(crate) struct Enemy;

#[derive(Component, Copy, Clone)]
pub(crate) struct Boss;

// bundle stores all relevant compnents of monsters 
#[derive(Bundle, Component, Copy, Clone)]
pub(crate) struct MonsterStats{
    // we need a &str that is texture of our monster
    // might need name as well
    pub typing: Element,
    pub lvl: Level,
    pub exp: Experience,
    pub hp: Health,
    pub vit: Vitality,
    pub stg: Strength,
    pub def: Defense,
    pub spd: Speed,
    pub acts: Actions,

    pub moves: Moves,

    pub slot: Slot,

    // pub enemy: Enemy,
}

// used for MonsterPartyBundle
#[derive(Component, Clone, Copy)]
pub(crate) struct SelectedMonster;
// Denotes a monster that is in our party
#[derive(Component, Clone, Copy)]
pub(crate) struct PartyMonster;
#[derive(Component)]
pub(crate) struct MonsterTotal(u8);
#[derive(Component)]
pub(crate) struct Fighting(bool);

// selected would highlight which monster is currently displayed; if SelectedMonster(0) then no display.
// fighting would inform of whether to spawn an enemy to fight as well.
// SelectedMonster != 0 and Fighting = false; then just show mon
// SelectedMonster != 0 and Fighting = true; then also display an Enemy monster
// monsters are added to the Party non-literally currently, MonsterPartyBundle keeps track of selected monster, total monsters had, and if there is a fight to be aware of.
#[derive(Bundle)]
pub(crate) struct MonsterPartyBundle{
    selected: SelectedMonster,
    monsters: MonsterTotal,
    fighting: Fighting,
}
// TODO: allow catching functionality by letting user choose after winning a battle to swap monster with one in party.

// Slot and Enemy may be redundant, but slot == 0 && Enemy == false allows for display of non-party, non-fighting monsters.
impl Default for MonsterStats {
    fn default() -> Self { MonsterStats {
        typing: rand::random(),
        lvl: Level {
			level: 1,
		},
        exp: Experience(0),
        hp: Health{
			max_health: 10,
			health: 10,
		},
        vit: Vitality(1),
        stg: Strength{atk: 2, crt: 25, crt_dmg: 2},
        def: Defense{def: 1, crt_res: 10},
        spd: Speed{spd: 1},
        acts: Actions(1),


        moves: Moves{known: 2, chosen: 0},

        slot: Slot(0),
    } }
}

// =========================================== HELPERS ===============================================

pub(crate) fn get_monster_sprite_for_type(elm: Element) -> String {
    match elm {
        Element::Scav => String::from("monsters/scav_monster.png"),
        Element::Growth => String::from("monsters/growth_monster.png"),
        Element::Ember => String::from("monsters/ember_monster.png"),
        Element::Flood => String::from("monsters/flood_monster.png"),
        Element::Rad => String::from("monsters/rad_monster.png"),
        Element::Robot => String::from("monsters/robot_monster.png"),
        Element::Clean => String::from("monsters/clean_monster.png"),
        Element::Filth => String::from("monsters/filth_monster.png"),
    }
}

// Need to apparently implement distribution for our
// elements enum to be able to pick randomly which type we want
impl Distribution<Element> for Standard {
    /// Randomly sample the element enum 
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Element {
        // Randomly generate a number from 0 to 7 then return an enum variant
        // corresponding to that.
        match rng.gen_range(0..=7) {
            0 => Element::Scav,
            1 => Element::Growth,
            2 => Element::Ember,
            3 => Element::Flood,
            4 => Element::Rad,
            5 => Element::Robot,
            6 => Element::Clean,
            _ => Element::Filth
        }
    }
}