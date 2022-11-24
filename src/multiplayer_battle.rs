#![allow(unused)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use rand::Rng;
use crate::game_client::{GameClient, self, PlayerType, Package, get_randomized_port};
use crate::monster::{Element};
use crate::{
	GameState
};
use std::str::from_utf8;
use std::{io, thread};
use std::net::{UdpSocket, Ipv4Addr};
use std::sync::mpsc::{Receiver, Sender, self};
use crate::camera::{MultCamera};
use crate::backgrounds::{
	WIN_H, WIN_W, 
	Tile
};

const MULT_BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";

#[derive(Component)]
pub(crate) struct MultBattleBackground;

#[derive(Component)]
pub(crate) struct MultMonster;

#[derive(Component)]
pub(crate) struct MultPlayerMonster;

// Unit structs to help identify the specific UI components for player's or enemy's monster health/level
// since there may be many Text components
#[derive(Component)]
pub (crate) struct MultPlayerHealth;

#[derive(Component)]
pub (crate) struct MultPlayerLevel;

#[derive(Component)]
pub(crate) struct MultBattleUIElement;

#[derive(Component)]
pub(crate) struct MultPlayerType;

pub struct MultBattlePlugin;

// Builds plugin for multiplayer battles
impl Plugin for MultBattlePlugin {
	fn build(&self, app: &mut App) {
		app
		.add_enter_system_set(GameState::MultiplayerBattle, 
            SystemSet::new()
                .with_system(setup_mult_battle)
                .with_system(setup_mult_battle_stats)
                // .with_system(generate_randomized_monster)
        )
		.add_system_set(ConditionSet::new()
			// Only run handlers on MultiplayerBattle state
			.run_in_state(GameState::MultiplayerBattle)
                //.with_system(generate_randomized_monster)
        .into())
		.add_exit_system(GameState::MultiplayerBattle, despawn_mult_battle);
	}
}

pub(crate) fn setup_mult_battle(mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, (With<Camera2d>, Without<MultCamera>)>,
    game_client: Res<GameClient>,
) { 

    cameras.for_each(|camera| {
		commands.entity(camera).despawn();
	});

    //creates camera for multiplayer battle background
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MultCamera);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load(MULT_BATTLE_BACKGROUND),
        transform: Transform::from_xyz(0., 0., 10.),
        ..default()
    })  
    .insert(MultBattleBackground);
}

pub(crate) fn setup_mult_battle_stats(
    mut commands: Commands, 
	asset_server: Res<AssetServer>,
    game_client: Res<GameClient>,
) {
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
    .insert(MultPlayerHealth)
    .insert(MultBattleUIElement);

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
                "0 here",
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
    .insert(MultPlayerLevel)
    .insert(MultBattleUIElement);
}

pub(crate) fn generate_randomized_monster() -> Result<Element, String> {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..=7) {
        0 => {
            info!("Number that was generated is 1");
             Ok(Element::Scav)
        },
        1 => {
            info!("Number that was generated is 2");
            Ok(Element::Growth)
        }
        2 => Ok(Element::Ember),
        3 => Ok(Element::Flood),
        4 => Ok(Element::Rad),
        5 => Ok(Element::Robot),
        6 => Ok(Element::Clean),
        7 => Ok(Element::Filth),
        _ => std::process::exit(0)
    }
}



fn despawn_mult_battle(mut commands: Commands,
	// camera_query: Query<Entity,  With<MenuCamera>>,
	// background_query: Query<Entity, With<MultMenuBackground>>,
    // mult_ui_element_query: Query<Entity, With<MultMenuUIElement>>
){

}
