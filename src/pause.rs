use bevy::{prelude::*, app::AppExit};
use iyes_loopless::prelude::*;
use crate::GameState;
use crate::camera::{MainCamera};
use crate::player::{Player};
use crate::backgrounds::{Tile, WIN_W};
use crate::start_menu::{TEXT_COLOR, NORMAL_BUTTON, PRESSED_BUTTON, HOVERED_BUTTON};

const BLANK: &str = "backgrounds/blank.png";

pub(crate) struct PausePlugin;

#[derive(Component)]
pub(crate) struct BlankBackground;

#[derive(Component)]
pub(crate) struct PauseUIElement;

#[derive(Component)]
pub(crate) struct QuitButton;

#[derive(Component)]
pub(crate) struct ResumeButton;

// #[derive(Component)]
// pub(crate) struct Text;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
		app
        .add_enter_system(GameState::Pause, setup_pause)
        .add_system_set(ConditionSet::new()
            .run_in_state(GameState::Pause)
                .with_system(handle_exit_pause)
                .with_system(quit_button_handler)
                .with_system(resume_button_handler)
            .into())
        .add_exit_system(GameState::Pause, despawn_pause);
	}
}

pub(crate) fn setup_pause(mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<&Transform, (With<Camera2d>, With<MainCamera>, Without<Player>, Without<Tile>)>, 
) {
    // Need camera's coordinates to draw blank screen
    if camera_query.is_empty() {
        error!("No camera found?");
        commands.insert_resource(NextState(GameState::Playing));
        return;
    }

    let camera = camera_query.single();

    commands.spawn_bundle(SpriteBundle {
		texture: asset_server.load(BLANK),
		transform: Transform::from_xyz(camera.translation.x, camera.translation.y, 1.),
		..default()
	}).insert(BlankBackground);
    commands
	.spawn_bundle(TextBundle::from_section(
			"PAUSED",
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
                left: Val::Px(560.0),
                ..default()
            },
            ..default()
        })
        .insert(PauseUIElement);  

    // QUIT GAME BUTTON
	commands
    .spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(300.0), Val::Px(65.0)),
            // center button
            margin: UiRect::all(Val::Auto),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            position: UiRect {
                bottom: Val::Px(125.),
                left: Val::Px(0.),
                ..default()
            },
        ..default()
    },
        color: NORMAL_BUTTON.into(),
        ..default()
    })
    .insert(QuitButton)
    .with_children(|parent| {
        parent.spawn_bundle(TextBundle::from_section(
            "QUIT GAME",
            TextStyle {
                font: asset_server.load("buttons/joystix monospace.ttf"),
                font_size: 40.0,
                color: TEXT_COLOR,
            },
        ));
    })
    .insert(PauseUIElement);

    // RESUME BUTTON
	commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(225.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(400.),
                    left: Val::Px((WIN_W * 0.825) / 2.),
                    ..default()
                },
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "RESUME",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: TEXT_COLOR,
                },
            ));
        })
        .insert(ResumeButton)
        .insert(PauseUIElement);
}

pub(crate) fn despawn_pause(mut commands: Commands,
    text_query: Query<Entity, With<Text>>,
    background_query: Query<Entity, With<BlankBackground>>,
    ui_elements: Query<Entity, With<PauseUIElement>>
) {
    // Despawn black background
    background_query.for_each(|backgrounds| {
        commands.entity(backgrounds).despawn();
    });

    // Despawn buttons
    ui_elements.for_each(|elem| {
        commands.entity(elem).despawn_recursive();
    })
}

/// Exit pause with the Esc key
fn handle_exit_pause(
    mut commands: Commands,
	input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        // Change back to start menu state
        commands.insert_resource(NextState(GameState::Playing));
    }
}

/// Quit the whole game by pressing Quit Game
pub (crate) fn quit_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<QuitButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
    mut exit: EventWriter<AppExit>
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                exit.send(AppExit);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

/// Resume game by pressing resume button
pub (crate) fn resume_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<ResumeButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
    mut exit: EventWriter<AppExit>
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                commands.insert_resource(NextState(GameState::Playing));
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}