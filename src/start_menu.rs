#[warn(unused_imports)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use crate::{
	GameState
};
use crate::camera::{MenuCamera};
use crate::player::{Player};
use crate::backgrounds::{
	WIN_H, WIN_W, 
	Tile
};

const MENU_BACKGROUND: &str = "backgrounds/start_screen.png";
const TEXT_COLOR: Color = Color::rgb(0.9,0.9,0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.75, 0.35, 0.35);

pub struct MainMenuPlugin;

#[derive(Component)]
pub(crate) struct MainMenuBackground;

#[derive(Component)]
pub(crate) struct StartButton;

#[derive(Component)]
pub(crate) struct CreditsButton;

#[derive(Component)]
pub(crate) struct MultiplayerButton;

#[derive(Component)]
pub(crate) struct StartMenuUIElement;

//Builds plugin called MainMenuPlugin
impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_enter_system(GameState::Start, setup_menu)
		.add_system_set(ConditionSet::new()
			// Only run handlers on Start state
			.run_in_state(GameState::Start)
				.with_system(start_button_handler)
				.with_system(credits_button_handler)
				.with_system(multiplayer_button_handler)
			.into())
		.add_exit_system(GameState::Start, despawn_start_menu);
	}
}

// Clears buttons from screen when ran
// Should be run after START button is pressed
fn despawn_start_menu(mut commands: Commands,
	button_query: Query<Entity, With<Button>>,
	camera_query: Query<Entity,  With<MenuCamera>>,
	background_query: Query<Entity, With<MainMenuBackground>>
){
	// Despawn buttons
	for b in button_query.iter() {
		commands.entity(b).despawn_recursive();
	}
	// Despawn cameras
	for c in camera_query.iter() {
		commands.entity(c).despawn();
	}
	// Despawn Main Menu Background
	for bckg in background_query.iter() {
		commands.entity(bckg).despawn();
	}
}

pub (crate) fn start_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<StartButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Start Game".to_string();
                *color = PRESSED_BUTTON.into();
                commands.insert_resource(NextState(GameState::StartPlaying));
            }
            Interaction::Hovered => {
                text.sections[0].value = "Start Game".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Start Game".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn credits_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<CreditsButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Credits".to_string();
                *color = PRESSED_BUTTON.into();
                commands.insert_resource(NextState(GameState::Credits));
            }
            Interaction::Hovered => {
                text.sections[0].value = "Credits".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Credits".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn multiplayer_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<MultiplayerButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Multiplayer".to_string();
                *color = PRESSED_BUTTON.into();
                // commands.insert_resource(NextState(GameState::StartPlaying));
            }
            Interaction::Hovered => {
                text.sections[0].value = "Multiplayer".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Multiplayer".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup_menu(mut commands: Commands,
	asset_server: Res<AssetServer>,
	cameras: Query<Entity, (With<Camera2d>, Without<MenuCamera>, Without<Player>, Without<Tile>)>
){ 
	cameras.for_each(|camera| {
		commands.entity(camera).despawn();
	});

	//creates camera for UI
	let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MenuCamera);

	commands.spawn_bundle(SpriteBundle {
		texture: asset_server.load(MENU_BACKGROUND),
		transform: Transform::from_xyz(0., 0., 0.),
		..default()
	})
	.insert(MainMenuBackground);
	

	// START BUTTON
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
			// 	position_type: PositionType::Absolute,
			// 	position: UiRect {
			// 	bottom: Val::Px(175.),
			// 	left: Val::Px((WIN_W * 0.8) / 2.),
			// 	..default()
			// },
			..default()
		},
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Start Game",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: TEXT_COLOR,
                },
            ));
        })
        .insert(StartButton)
        .insert(StartMenuUIElement);
	

	// CREDITS BUTTON
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
				bottom: Val::Px(170.),
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
			"Credits",
			TextStyle {
				font: asset_server.load("buttons/joystix monospace.ttf"),
				font_size: 40.0,
				color: TEXT_COLOR,
			},
		));
	})
	.insert(CreditsButton)
	.insert(StartMenuUIElement);


	// MULTIPLAYER BUTTON
	commands
	.spawn_bundle(ButtonBundle {
		style: Style {
			size: Size::new(Val::Px(325.0), Val::Px(65.0)),
			// center button
			margin: UiRect::all(Val::Auto),
			// horizontally center child text
			justify_content: JustifyContent::Center,
			// vertically center child text
			align_items: AlignItems::Center,
			position_type: PositionType::Absolute,
			position: UiRect {
				bottom: Val::Px(250.),
				left: Val::Px((WIN_W * 0.75) / 2.),
				..default()
			},
			..default()
		},
		color: NORMAL_BUTTON.into(),
		..default()
	})
	.with_children(|parent| {
		parent.spawn_bundle(TextBundle::from_section(
			"Multiplayer",
			TextStyle {
				font: asset_server.load("buttons/joystix monospace.ttf"),
				font_size: 40.0,
				color: TEXT_COLOR,
			},
		));
	})
	.insert(MultiplayerButton)
	.insert(StartMenuUIElement);
}