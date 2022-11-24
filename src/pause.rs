use bevy::{prelude::*};
use iyes_loopless::prelude::*;
use crate::GameState;
use crate::camera::{MainCamera};
use crate::player::{Player};
use crate::backgrounds::{Tile};

const BLANK: &str = "backgrounds/blank.png";

pub(crate) struct PausePlugin;

#[derive(Component)]
pub(crate) struct BlankBackground;

#[derive(Component)]
pub struct Text;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
		app
        .add_enter_system(GameState::Pause, setup_pause)
        .add_system_set(ConditionSet::new()
            .run_in_state(GameState::Pause)
                .with_system(handle_exit_pause)
            .into())
        .add_exit_system(GameState::Pause, despawn_pause);
	}
}

pub(crate) fn setup_pause(mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.spawn_bundle(SpriteBundle {
		texture: asset_server.load(BLANK),
		transform: Transform::from_xyz(0., 0., 0.),
		..default()
	}).insert(BlankBackground);
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
    text_query: Query<Entity, With<Text>>,
    background_query: Query<Entity, With<BlankBackground>>
) {
   
    // Despawn text
    text_query.for_each(|text| {
        commands.entity(text).despawn();
    });

    background_query.for_each(|backgrounds| {
        commands.entity(backgrounds).despawn();
    });
}

fn handle_exit_pause(
    mut commands: Commands,
	input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        // Change back to start menu state
        commands.insert_resource(NextState(GameState::Playing));
    }
}