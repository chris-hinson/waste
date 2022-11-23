use bevy::{prelude::*};
use iyes_loopless::prelude::*;
use crate::GameState;
use crate::camera::{HelpCamera};
use crate::player::{Player};
use crate::backgrounds::{Tile};

pub(crate) struct HelpPlugin;

#[derive(Component)]
pub struct Text;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
		app
        .add_enter_system(GameState::Help, setup_help)
        .add_system_set(ConditionSet::new()
            // Run these systems only when in Credits states
            .run_in_state(GameState::Help)
                .with_system(handle_exit_help)
            .into())
        .add_exit_system(GameState::Help, despawn_help)
        .add_exit_system(GameState::Help, crate::teardown);
	}
}

pub(crate) fn setup_help(mut commands: Commands,
    cameras: Query<Entity, (With<Camera2d>, Without<HelpCamera>, Without<Player>, Without<Tile>)>,
    asset_server: Res<AssetServer>
) {
    // Despawn all non-help cameras
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // Spawn help camera
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(HelpCamera);
    commands
	.spawn_bundle(TextBundle::from_section(
			"HELP",
			TextStyle {
				font: asset_server.load("buttons/joystix monospace.ttf"),
				font_size: 40.0,
				color: Color::WHITE,
			},
        ))
        .insert(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(700.0),
                left: Val::Px(600.0),
                ..default()
            },
            ..default()
        })
        .insert(Text);  

    commands
	.spawn_bundle(TextBundle::from_section(
			"MOVEMENT CONTROLS",
			TextStyle {
				font: asset_server.load("buttons/joystix monospace.ttf"),
				font_size: 35.0,
				color: Color::WHITE,
			},
        ))
        .insert(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(600.0),
                left: Val::Px(500.0),
                ..default()
            },
            ..default()
        })
        .insert(Text);

    commands
	.spawn_bundle(TextBundle::from_section(
			"W to move up, S to move down, A to move left, D to move right",
			TextStyle {
				font: asset_server.load("buttons/joystix monospace.ttf"),
				font_size: 30.0,
				color: Color::WHITE,
			},
        ))
        .insert(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(500.0),
                left: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .insert(Text);

    commands
	.spawn_bundle(TextBundle::from_section(
			"BATTLE CONTROLS",
			TextStyle {
				font: asset_server.load("buttons/joystix monospace.ttf"),
				font_size: 35.0,
				color: Color::WHITE,
			},
        ))
        .insert(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(400.0),
                left: Val::Px(500.0),
                ..default()
            },
            ..default()
        })
        .insert(Text);

        commands
	.spawn_bundle(TextBundle::from_section(
			"A to attack, D to defend, q to quit",
			TextStyle {
				font: asset_server.load("buttons/joystix monospace.ttf"),
				font_size: 30.0,
				color: Color::WHITE,
			},
        ))
        .insert(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(300.0),
                left: Val::Px(300.0),
                ..default()
            },
            ..default()
        })
        .insert(Text);
}

pub(crate) fn despawn_help(mut commands: Commands,
	camera_query: Query<Entity,  With<HelpCamera>>,
    text_query: Query<Entity, With<Text>>
) {
    // Despawn credits camera
    camera_query.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // Despawn text
    text_query.for_each(|text| {
        commands.entity(text).despawn();
    });
}

fn handle_exit_help(
    mut commands: Commands,
	input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Escape) {
        // Change back to start menu state
        commands.insert_resource(NextState(GameState::Start));
    }
}
