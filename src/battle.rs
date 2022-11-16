#![allow(unused)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use crate::monster::{MonsterStats, Enemy, Actions, Fighting, SelectedMonster, Health, Level, Strength, Defense, Move, Moves, get_monster_sprite_for_type, Element, Boss};
use crate::{GameState, player};
use crate::game_client::{GameClient, Package};
use std::net::UdpSocket;
use std::sync::mpsc::{Sender, Receiver, self};
use std::thread;
use crate::backgrounds::Tile;
use crate::camera::{MainCamera, MenuCamera, SlidesCamera};
use crate::player::Player;
use crate::world::GameProgress;
use rand::*;

const BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";
const ENEMY_MONSTER: &str = "monsters/clean_monster.png";
const MONSTER: &str = "monsters/stickdude.png";

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.75, 0.35, 0.35);


#[derive(Component)]
pub(crate) struct BattleBackground;

#[derive(Component)]
pub(crate) struct Monster;

#[derive(Component)]
pub(crate) struct PlayerMonster;

#[derive(Component)]
pub (crate) struct EnemyMonster;

// Unit structs to help identify the specific UI components for player's or enemy's monster health/level
// since there may be many Text components
#[derive(Component)]
pub (crate) struct PlayerHealth;

#[derive(Component)]
pub (crate) struct EnemyHealth;

#[derive(Component)]
pub (crate) struct PlayerLevel;

#[derive(Component)]
pub (crate) struct EnemyLevel;

#[derive(Component)]
pub(crate) struct AbortButton;

#[derive(Component)]
pub(crate) struct AttackButton;

#[derive(Component)]
pub(crate) struct DefendButton;

#[derive(Component)]
pub(crate) struct BattleUIElement;

struct UiAssets{
	font: Handle<Font>,
	button: Handle<Image>,
	button_pressed: Handle<Image>,
}

pub(crate) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system_set(GameState::Battle, 
                SystemSet::new()
                    .with_system(setup_battle)
                    .with_system(setup_battle_stats)
                    .with_system(abort_button)
                    .with_system(attack_button)
                    .with_system(defend_button)
                    // .with_system(spawn_player_monster)
                    // .with_system(spawn_enemy_monster)
                )
            .add_system_set(ConditionSet::new()
                // Run these systems only when in Battle state
                .run_in_state(GameState::Battle)
                    // addl systems go here
                    .with_system(spawn_player_monster)
                    .with_system(spawn_enemy_monster)
                    .with_system(abort_button_handler)
                    .with_system(attack_button_handler)
                    .with_system(defend_button_handler)
                    .with_system(update_battle_stats)                
                .into())
            .add_exit_system(GameState::Battle, despawn_battle);
    }
}

macro_rules! end_battle {
    ($commands:expr, $game_progress:expr, $my_monster:expr, $enemy_monster:expr) => {
        // remove the monster from the enemy stats
        $game_progress.enemy_stats.remove(&$enemy_monster.5);
        // reset selected monster back to the first one in our bag
        let first_monster = $game_progress.monster_id_entity.get(&1).unwrap().clone();
        $commands.entity($my_monster.5).remove::<SelectedMonster>();
        $commands.entity(first_monster).insert(SelectedMonster);
        // the battle is over, remove enemy from monster anyways
        $commands.entity($enemy_monster.5).remove::<Enemy>();
        $commands.insert_resource(NextState(GameState::Playing));  
    }
}

macro_rules! monster_level_up {
    ($commands:expr, $game_progress:expr, $my_monster:expr, $up_by:expr) => {
        info!("your monster level up!");
        let mut stats = $game_progress.monster_entity_to_stats.get_mut(&$my_monster).unwrap();
        stats.lvl.level += 1 * $up_by;
        stats.hp.max_health += 10 * $up_by;
        stats.hp.health = stats.hp.max_health as isize;
        stats.stg.atk += 2 * $up_by;
        stats.stg.crt += 5 * $up_by;
        stats.def.def += 1 * $up_by;
        // we have to remove the old stats and add the new one
        // because we cannot change the stats in place
        $commands.entity($my_monster).remove::<MonsterStats>();
        $commands.entity($my_monster).insert_bundle(stats.clone());
    };
}

pub(crate) fn setup_battle(mut commands: Commands,
                           asset_server: Res<AssetServer>,
                           cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>)>,
                            game_client: Res<GameClient>,
) { 

    //let temp 
    if cameras.is_empty() {
        error!("No spawned camera...?");
    } else{

    }
    let (ct, _) = cameras.single();

    // commands.spawn_bundle(MonsterBundle {
    //     ..Default::default()
    // }).insert(SelectedMonster);
    // commands.spawn_bundle(MonsterBundle {
    //     ..Default::default()
    // }).insert(Enemy);

    // Backgrounds overlayed on top of the game world (to prevent the background
    // from being despawned and needing regenerated by WFC).
    // Main background is on -1, so layer this at 0.
    // Monsters can be layered at 1. and buttons/other UI can be 2.
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load(BATTLE_BACKGROUND),
        transform: Transform::from_xyz(ct.translation.x, ct.translation.y, 0.),
        ..default()
    })
        .insert(BattleBackground);
} 

pub(crate) fn setup_battle_stats(mut commands: Commands, 
	asset_server: Res<AssetServer>,
    mut set: ParamSet<(
        Query<&mut Level, With<SelectedMonster>>,
        Query<&mut Level, With<Enemy>>,
    )>, 
){

    let mut my_lvl = 0;
    let mut enemy_lvl = 0;
    for mut my_monster in set.p0().iter_mut() {
        my_lvl = my_monster.level;
    }


    for mut enemy_monster in set.p1().iter_mut() {
        enemy_lvl = enemy_monster.level;
    }


    commands.spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // health header for player's monster
                TextSection::new(
                    "Health:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // health of player's monster
                // TextSection::new(
                //     my_hp.to_string(),
                //     TextStyle {
                //         font: asset_server.load("buttons/joystix monospace.ttf"),
                //         font_size: 40.0,
                //         color: Color::BLACK,
                //     },
                // )
                // health of player's monster
                TextSection::from_style(TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::BLACK,
                }),
            ])
            .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(5.0),
                        left: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(PlayerHealth)
        .insert(BattleUIElement);

        commands.spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // level header for player's monster
                TextSection::new(
                    "Level:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // level of player's monster
                TextSection::new(
                    my_lvl.to_string(),
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                )
            ])
            .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(40.0),
                        left: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(PlayerLevel)
        .insert(BattleUIElement);

        commands.spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // health header for enemy's monster
                TextSection::new(
                    "Health:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // health of enemy's monster
                TextSection::from_style(TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::BLACK,
                }),
            ])
            .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(5.0),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        //.insert(MonsterBundle::default())
        .insert(EnemyHealth)
        .insert(BattleUIElement);

        commands.spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // level header for player's monster
                TextSection::new(
                    "Level:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // level of player's monster
                TextSection::new(
                	enemy_lvl.to_string(),
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                )
            ])
            .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(40.0),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(EnemyLevel)
        .insert(BattleUIElement);

}

pub(crate) fn update_battle_stats(mut commands: Commands, 
	asset_server: Res<AssetServer>,
    mut set: ParamSet<(
        Query<&mut Health, With<SelectedMonster>>,
        Query<&mut Health, With<Enemy>>,
    )>, 
    mut enemy_health_text_query: Query<&mut Text, (With<EnemyHealth>, Without<PlayerHealth>)>,
    mut player_health_text_query: Query<&mut Text, (With<PlayerHealth>, Without<EnemyHealth>)>
){

    let mut my_health = 0;
    let mut enemy_health = 0;
    for mut my_monster in set.p0().iter_mut() {
        my_health = my_monster.health;
    }

    for mut enemy_monster in set.p1().iter_mut() {
        enemy_health = enemy_monster.health;
    }

    for mut text in &mut enemy_health_text_query {
        text.sections[1].value = format!("{}", enemy_health.to_string());
    }

    for mut text in &mut player_health_text_query {
        text.sections[1].value = format!("{}", my_health.to_string());
    }

}

// pub(crate) fn update_battle_stats(
//     mut query: Query<&mut Text, With<EnemyHealth>>,
//     mut monster_query: Query<(&mut MonsterBundle)>,
// ) {
//     let mut monster = monster_query.single_mut();
//     for mut text in &mut query {
//         text.sections[1].value = format!("{}", &monster.hp.health.to_string());
//     }
// }


pub(crate) fn spawn_player_monster(mut commands: Commands, 
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>)>,
    selected_monster_query: Query<(&Element, Entity), (With<SelectedMonster>, Without<Enemy>)>,
) {
    if cameras.is_empty() {
        error!("No spawned camera...?");
        return;
    }

    if selected_monster_query.is_empty() {
        error!("No selected monster...?");
        return;
    }

    let (ct, _) = cameras.single();

    // why doesn't this update
    let (selected_type, selected_monster) = selected_monster_query.single();

    commands
    .entity(selected_monster)
    .insert_bundle(
        SpriteBundle {
            sprite: Sprite {
                flip_y: false,  // flips our little buddy, you guessed it, in the y direction
                flip_x: true,   // guess what this does
                ..default()
            },
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x - 400., ct.translation.y - 100., 1.), 
            ..default()
    })
    .insert(PlayerMonster)
    .insert(Monster);

}


pub(crate) fn spawn_enemy_monster(mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>)>,
    selected_type_query: Query<(&Element), (Without<SelectedMonster>, With<Enemy>)>,
) {

    if cameras.is_empty() 
    {
        error!("No spawned camera...?");
        return;
    }

    if selected_type_query.is_empty() {
        error!("No selected monster...?");
        return;
    }

    let selected_type = selected_type_query.single();

    let (ct, _) = cameras.single();

    // This spawns a new monster that has nothing to do with battle
    // The one fore battle is spawned in the player.rs when we collide with a monster tile
    // let monster_info = MonsterStats {
    //     ..default()
    // };


    // let sprite_string = &get_monster_sprite_for_type(monster_info.clone().typing);

    commands.spawn_bundle(
        SpriteBundle {
            // texture: asset_server.load(sprite_string),
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x + 400., ct.translation.y - 100., 1.), 
            ..default()
    })
        .insert(EnemyMonster)
        .insert(Monster);
        // .insert(monster_info.clone());
}

// handles abort button for multplayer battles 
pub (crate) fn mult_abort_handler (
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<AbortButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Abort".to_string();
                *color = PRESSED_BUTTON.into();
                commands.insert_resource(NextState(GameState::Start));
            }
            Interaction::Hovered => {
                text.sections[0].value = "Abort".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Abort".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn abort_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<AbortButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
    mut enemy_monster: 
        Query<Entity, (Without<SelectedMonster>, With<Enemy>)>,

) { 
    let mut em = enemy_monster.single_mut();

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Abort".to_string();
                *color = PRESSED_BUTTON.into();
                // This is gonna cause us problems as is, until we modify
                // states so that the initial transition from Start -> StartPlaying (a new state)
                // is the only one that spawns the world. In this paradigm,
                // it will regenerate the whole world as if it just started.
                commands.entity(em).remove::<Enemy>();
                commands.insert_resource(NextState(GameState::Playing));
            }
            Interaction::Hovered => {
                text.sections[0].value = "Abort".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Abort".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn attack_button_handler (
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<AttackButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
    mut my_monster: 
        Query<(&mut Level, &mut Health, &mut Strength, &mut Defense, &mut Moves, Entity), 
        (With<SelectedMonster>, Without<Enemy>)>,
    mut enemy_monster: 
        Query<(&mut Level, &mut Health, &mut Strength, &mut Defense, &mut Moves, Entity, Option<&Boss>), 
        (Without<SelectedMonster>, With<Enemy>)>,
    mut game_progress: ResMut<GameProgress>,
 ) {

    if(my_monster.is_empty() || enemy_monster.is_empty()) {
        info!("Monsters are missing!");
        commands.insert_resource(NextState(GameState::Playing));
    }

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Attack".to_string();
                *color = PRESSED_BUTTON.into();
                
                let mut pm = my_monster.single_mut();
                let mut em = enemy_monster.single_mut();
                // Actions: 
                // 0: attack 1: defend: 2: heal: 3: customize yourself
                let mut enemy_action = rand::thread_rng().gen_range(0..=1);
                info!("You attack!");
                if enemy_action == 1 {
                    info!("Enemy defends!")
                } else {
                    info!("Enemy attacks!")
                }
                let turn_result = calculate_damage(&pm.2, &pm.3, 0, &em.2, &em.3, enemy_action);

                pm.1.health -= turn_result.1;
                em.1.health -= turn_result.0;

                if em.1.health <= 0 {
                    info!("Enemy monster defeated.");
                    // at this point this monster is already "ours", we just need to register is with the resource
                    // get the stats from the monster
                    let new_monster_stats = game_progress.enemy_stats.get(&em.5).unwrap().clone();
                    // remove the monster from the enemy stats
                    game_progress.enemy_stats.remove(&em.5);
                    // add the monster to the monster bag
                    game_progress.new_monster(em.5, new_monster_stats);
                    // TODO: see the discrepancy between the type we see and the type we get
                    info!("new member type: {:?}", game_progress.monster_entity_to_stats.get(&em.5).unwrap().typing);
                    // update game progress
                    // check for boss
                    if em.6.is_some() {
                        info!("Boss defeated!");
                        game_progress.win_boss();
                        // if boss level up twice
                        monster_level_up!(commands, game_progress, pm.5, 2);
                        commands.entity(em.5).remove::<Boss>();
                    } else {
                        game_progress.win_battle();
                        // if not boss level up once
                        monster_level_up!(commands, game_progress, pm.5, 1);
                    }
                    end_battle!(commands, game_progress, pm, em);
                } else if pm.1.health <= 0 {
                    let next_monster = game_progress.next_monster(pm.5);
                    if next_monster.is_none() {
                        info!("Your monster was defeated.");
                        end_battle!(commands, game_progress, pm, em);
                    } else {
                        info!("Your monster was defeated. Switching to next monster.");
                        commands.entity(pm.5).remove::<SelectedMonster>();
                        commands.entity(pm.5).remove_bundle::<SpriteBundle>();
                        commands.entity(pm.5).remove::<PlayerMonster>();
                        commands.entity(pm.5).remove::<Monster>();
                        commands.entity(*next_monster.unwrap()).insert(SelectedMonster); 
                    }   
                }

            }
            Interaction::Hovered => {
                text.sections[0].value = "Attack".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Attack".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn defend_button_handler (
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<DefendButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
    mut my_monster: 
        Query<(&mut Level, &mut Health, &mut Strength, &mut Defense, &mut Moves), (With<SelectedMonster>, Without<Enemy>)>,
    mut enemy_monster: 
        Query<(&mut Level, &mut Health, &mut Strength, &mut Defense, &mut Moves, Entity), (Without<SelectedMonster>, With<Enemy>)>,
) {

    if(my_monster.is_empty() || enemy_monster.is_empty()) {
        info!("Monsters are missing!");
        commands.insert_resource(NextState(GameState::Playing));
    }

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Defend".to_string();
                *color = PRESSED_BUTTON.into();

                // let mut pm = my_monster.single_mut();
                // let mut em = enemy_monster.single_mut();
                // I just realized that we don't need to do anything here, at least for how this is set up now.
                let mut enemy_action = rand::thread_rng().gen_range(0..=1);
                info!("You defend!");
                if enemy_action == 1 {
                    info!("Enemy defends!")
                } else {
                    info!("Enemy attacks!")
                }
                // let turn_result = calculate_damage(&pm.2, &pm.3, 1, &em.2, &em.3, enemy_action);

                // pm.1.health -= turn_result.1;
                // em.1.health -= turn_result.0;

                // if em.1.health <= 0 {
                //     info!("Enemy monster defeated");
                //     commands.entity(em.5).remove::<Enemy>();
                //     // pm.1.health = pm.1.max_health as isize;
                //     commands.insert_resource(NextState(GameState::Playing));         
                // } else if pm.1.health <= 0 {
                //     info!("Your monster was defeated");
                //     commands.entity(em.5).remove::<Enemy>();
                //     // pm.1.health = pm.1.max_health as isize;
                //     commands.insert_resource(NextState(GameState::Playing));     
                // }
            }
            Interaction::Hovered => {
                text.sections[0].value = "Defend".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Defend".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub(crate) fn abort_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(100.0),
                    left: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Abort",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .insert(AbortButton)
        .insert(BattleUIElement);
}

pub(crate) fn attack_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(100.0),
                    left: Val::Px(325.0),
                    ..default()
                },
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Attack",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .insert(AttackButton)
        .insert(BattleUIElement);
}

pub(crate) fn defend_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(100.0),
                    left: Val::Px(550.0),
                    ..default()
                },
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Defend",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .insert(DefendButton)
        .insert(BattleUIElement);
}

pub(crate) fn despawn_battle(mut commands: Commands,
    background_query: Query<Entity, With<BattleBackground>>,
    monster_query: Query<Entity, With<Monster>>,
    battle_ui_element_query: Query<Entity, With<BattleUIElement>>
) {
    if background_query.is_empty() 
    {
        error!("background is here!");
    }

   background_query.for_each(|background| {
        commands.entity(background).despawn();
        info!("got here");
   });

   if monster_query.is_empty() 
   {
        error!("monsters are here!");
   }

   monster_query.for_each(|monster| {
        commands.entity(monster)                       
        .remove_bundle::<SpriteBundle>()
        .remove::<PlayerMonster>()
        .remove::<EnemyMonster>()
        .remove::<Monster>();
   });


   if battle_ui_element_query.is_empty() 
    {
    error!("ui elements are here!");
    }

   battle_ui_element_query.for_each(|battle_ui_element| {
        commands.entity(battle_ui_element).despawn_recursive();
   });

}


fn calculate_damage(player_stg: &Strength, player_def: &Defense, player_action: usize, 
    enemy_stg: &Strength, enemy_def: &Defense, enemy_action: usize) -> (isize, isize) {
    if (player_action == 1 || enemy_action == 1) {
        // if either side defends this turn will not have any damage on either side
        return (0, 0);
    }
    // More actions can be added later, we can also consider decoupling the actions from the damage
    let mut result = (0,0);
    // player attacks
    // If our attack is less than the enemy's defense, we do 0 damage
    if player_stg.atk <= enemy_def.def {
        result.0 = 0;
    } else {
        // if we have damage, we do that much damage
        // I've only implemented crits for now, dodge and element can follow
        result.0 = player_stg.atk - enemy_def.def;
        if player_stg.crt <= enemy_def.crt_res {
            result.0 = result.0;
        } else {
            // calculate crit chance and apply crit damage
            let mut crit_chance = player_stg.crt - enemy_def.crt_res;
            let crit = rand::thread_rng().gen_range(0..=100);
            if crit <= crit_chance {
                result.0 *= player_stg.crt_dmg;
            }
        }
    }
    // same for enemy
    if enemy_stg.atk <= player_def.def {
        result.1 = 0;
    } else {
        result.1 = enemy_stg.atk - player_def.def;
        if enemy_stg.crt <= player_def.crt_res {
            result.1 = result.1;
        } else {
            let mut crit_chance = enemy_stg.crt - player_def.crt_res;
            let crit = rand::thread_rng().gen_range(0..=100);
            if crit <= crit_chance {
                result.1 *= enemy_stg.crt_dmg;
            }
        }
    }

    return (result.0 as isize, result.1 as isize)
}