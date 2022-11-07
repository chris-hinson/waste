// current implementation focuses on getting all essential data for monsters in game, will need further optimizations in here and functions later.

use bevy::{prelude::*};
use iyes_loopless::prelude::*;
use crate::GameState;
use crate::camera::{SlidesCamera};
use crate::player::{Player};
use crate::backgrounds::{Tile};
use rand::seq::SliceRandom;

pub(crate) struct MonsterPlugin;
// unused at the moment
pub(crate) const BASIC_ENEMY: &str = "monsters/clean_monster.png";

// Elemental types
#[derive(Component, Copy, Clone)]
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
	pub max_level: u16,
	pub level: u16,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct Experience(u16);
#[derive(Component, Copy, Clone)]
pub(crate) struct Health {
	pub max_health: u16,
	pub health: u16,
}
#[derive(Component, Copy, Clone)]
pub(crate) struct Vitality(u8);
#[derive(Component, Copy, Clone)]
pub(crate) struct Strength(u8);
#[derive(Component, Copy, Clone)]
pub(crate) struct Defense(u8);
#[derive(Component, Copy, Clone)]
pub(crate) struct Speed(u8);
#[derive(Component, Copy, Clone)]
// keeps track of the number of Actions per Turn a monster has (1-3 for now) (4 for bosses)
pub(crate) struct Actions(u8);
// to keep track of the Number of Known Moves a monster has (1-4, has to know 1)
#[derive(Component, Copy, Clone)]
pub(crate) struct NumMoves(u8);
// keeps track of which slot in the party a monster is in. (0 by default means not in the party)
#[derive(Component, Copy, Clone)]
pub(crate) struct Slot(u8);

// tells you if a monster is an enemy or friend (in-party true)
#[derive(Component, Copy, Clone)]
pub(crate) struct Enemy(bool);


// bundle stores all relevant compnents of monsters 
#[derive(Bundle, Component, Copy, Clone)]
pub(crate) struct MonsterBundle{
    _typing: Element,
    pub lvl: Level,
    _exp: Experience,
    pub hp: Health,
    _vit: Vitality,
    _stg: Strength,
    _def: Defense,
    _spd: Speed,
    _acts: Actions,

    _moves: NumMoves,

    _slot: Slot,

    _enemy: Enemy,
}

// used for MonsterPartyBundle
#[derive(Component)]
pub(crate) struct SelectedMonster(u8);
#[derive(Component)]
pub(crate) struct MonsterTotal(u8);
#[derive(Component)]
pub(crate) struct Fighting(bool);

// selected would highlight which monster is currently displayed; if SelectedMonster(0) then no display.
// fighting would inform of whether to spawn an enemy to fight as well.
// SelectedMonster != 0 and Fighting = false; then just show mon
// SelectedMonster !=0 and Fighting = true; then also display an Enemy monster
// monsters are added to the Party non-literally currently, MonsterPartyBundle keeps track of selected monster, total monsters had, and if there is a fight to be aware of.
#[derive(Bundle)]
pub(crate) struct MonsterPartyBundle{
    _selected: SelectedMonster,
    _monsters: MonsterTotal,
    _fighting: Fighting,
}
// TODO: allow catching functionality by letting user choose after winning a battle to swap monster with one in party.

// Slot and Enemy may be redundant, but slot == 0 && Enemy == false allows for display of non-party, non-fighting monsters.
impl Default for MonsterBundle {
    fn default() -> Self { MonsterBundle {
        _typing: Element::Clean,
        lvl: Level {
			max_level: 10,
			level: 1,
		},
        _exp: Experience(0),
        hp: Health{
			max_health: 10,
			health: 10,
		},
        _vit: Vitality(1),
        _stg: Strength(1),
        _def: Defense(1),
        _spd: Speed(1),
        _acts: Actions(1),


        _moves: NumMoves(2),

        _slot: Slot(0),

        _enemy: Enemy(true),
    } }
}
// TODO: implement randomized monster initialization, may be difficult to balance how leveling changes start values.

// default is that you're fighting 
impl Default for MonsterPartyBundle {
    fn default() -> Self { MonsterPartyBundle {
        _selected: SelectedMonster(0),
        _monsters: MonsterTotal(1),
        _fighting: Fighting(false),
    }
} }

// TODO: save the party to a file somehow to be able to revisit your party when re-opening the game.
impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App){
        app.add_startup_system(spawn_initial_party);
    }
}

// player has to have a monster in party, currently also spawns an enemy
// implementation of Enemy Monsters needs further consideration, maybe switch to a Component Header to reduce Components down.

fn spawn_initial_party(mut commands: Commands
    ){
        commands.spawn()
            .insert_bundle(MonsterPartyBundle { ..Default::default()})
            .insert_bundle(MonsterBundle{_slot: Slot(1), _enemy: Enemy(false), ..Default::default() })
            .insert_bundle(MonsterBundle{_slot: Slot(0), _enemy: Enemy(true), ..Default::default() });

            // below is additional consideration into nesting bundles to allow for MonsterPartyBundle to also store the MonsterBundles but i was having issues, right now it just tracks the data for all monsterBundles spawned in game.
            
            //.with_children(|parent| {parent.spawn_bundle(MonsterBundle{ _slot: Slot(1), _enemy: Enemy(false), ..Default::default() });
            //parent.spawn_bundle(MonsterBundle{ _lvl: Level(0), ..Default::default()});});
}   