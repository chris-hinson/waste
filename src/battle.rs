#[warn(unused_imports)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use crate::{GameState};
use crate::backgrounds::Tile;
use crate::camera::{MainCamera, MenuCamera, SlidesCamera};
use crate::player::Player;

const BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";
const ENEMY_MONSTER: &str = "characters/clean_monster.png";
const MONSTER: &str = "characters/stickdude.png";


#[derive(Component)]
pub(crate) struct BattleBackground;

#[derive(Component)]
pub(crate) struct Monster;

#[derive(Component)]
pub (crate) struct EnemyMonster;

// Unit structs to help identify the specifi UI component for the player, since there may be many Text components
#[derive(Component)]
struct PlayerHealth;

// Unit struct to help identify Enemy Health UI component, since there may be many Text components
#[derive(Component)]
struct EnemyHealth;

#[derive(Component)]
struct PlayerLevel;

#[derive(Component)]
struct EnemyLevel;

pub(crate) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::Battle, setup_battle)
            .add_system_set(ConditionSet::new()
                // Run these systems only when in Battle state
                .run_in_state(GameState::Battle)
                    // addl systems go here
                .into())
            .add_exit_system(GameState::Battle, despawn_battle);
    }
}

pub(crate) fn setup_battle(mut commands: Commands,
                           asset_server: Res<AssetServer>,
                           cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>,
                            Without<Player>, Without<Tile>)>
) {
    if cameras.is_empty() {
        error!("No spawned camera...?");
      }
      let (ct, _) = cameras.single();
  
    //   commands.spawn_bundle(SpriteBundle {
    //       texture: asset_server.load(BATTLE_BACKGROUND),
    //       transform: Transform::from_xyz(ct.translation.x, ct.translation.y, ct.translation.z),
    //       ..default()
    //   })
    //       .insert(BattleBackground);
}

pub(crate) fn battle_stats(mut commands: Commands, asset_server: Res<AssetServer>) 
{
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
                TextSection::new(
                    "10",
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
                        top: Val::Px(5.0),
                        left: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(PlayerHealth);

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
                    "1",
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
        .insert(PlayerLevel);

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
                TextSection::new(
                    "20",
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
                        top: Val::Px(5.0),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(EnemyHealth);

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
                    "1",
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
        .insert(EnemyLevel);

}

pub(crate) fn spawn_battle_sprite(mut commands: Commands, 
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>)>,
    //mut player_query: Query<Entity, With<Player>>
) {
    //let mut player = player_query.single_mut();
    if cameras.is_empty() 
    {
        error!("No spawned camera...?");
    }

      let (ct, _) = cameras.single();

    commands.spawn_bundle(
        SpriteBundle {
        texture: asset_server.load(MONSTER),
        transform: Transform::from_xyz(ct.translation.x - 500., ct.translation.y, ct.translation.z), 
        ..default()
    })
        .insert(Monster);

}


pub(crate) fn spawn_monster(mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>)>
) {

    if cameras.is_empty() 
    {
        error!("No spawned camera...?");
    }

      let (ct, _) = cameras.single();

    commands.spawn_bundle(
        SpriteBundle {
        texture: asset_server.load(ENEMY_MONSTER),
        transform: Transform::from_xyz(ct.translation.x + 500., ct.translation.y, ct.translation.z), 
        ..default()
    })
        .insert(EnemyMonster);
}


pub(crate) fn despawn_battle(mut commands: Commands,
    camera_query: Query<Entity,  With<MainCamera>>,
    background_query: Query<Entity, With<Tile>>,
) {
   camera_query.for_each(|camera| {
       commands.entity(camera).despawn();
   });

   background_query.for_each(|background| {
       commands.entity(background).despawn();
   });
}