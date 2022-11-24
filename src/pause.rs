use bevy::{prelude::*};
use iyes_loopless::prelude::*;
use crate::GameState;
use crate::camera::{PauseCamera, MainCamera};
use crate::player::{Player};
use crate::backgrounds::{Tile};

pub(crate) struct PausePlugin;

#[derive(Component)]
pub struct Text;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
		app
        .add_enter_system(GameState::Pause, setup_pause)
        .add_system_set(ConditionSet::new()
            // Run these systems only when in Credits states
            .run_in_state(GameState::Pause)
                .with_system(handle_exit_pause)
            .into())
        .add_exit_system(GameState::Pause, despawn_pause)
        .add_exit_system(GameState::Pause, crate::teardown);
	}
}

pub(crate) fn setup_pause(mut commands: Commands,
    cameras: Query<Entity, (With<MainCamera>, Without<PauseCamera>)>,
    asset_server: Res<AssetServer>
) {
    // Despawn 
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // Spawn 
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(PauseCamera);
    commands
	.spawn_bundle(TextBundle::from_section(
			"PAUSE",
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
}

pub(crate) fn despawn_pause(mut commands: Commands,
	camera_query: Query<Entity,  With<PauseCamera>>,
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

fn handle_exit_pause(
    mut commands: Commands,
	input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Escape) {
        // Change back to start menu state
        commands.insert_resource(NextState(GameState::Playing));
    }
}
