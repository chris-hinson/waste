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

const MULT_MENU_BACKGROUND: &str = "backgrounds/multiplayer_screen.png";
const TEXT_COLOR: Color = Color::rgb(0.9,0.9,0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.75, 0.35, 0.35);

pub struct MultMenuPlugin;

#[derive(Component)]
pub(crate) struct MultMenuBackground;

#[derive(Component)]
pub(crate) struct MultOptionsText;

#[derive(Component)]
pub(crate) struct HostButton;

#[derive(Component)]
pub(crate) struct ClientButton;

#[derive(Component)]
pub(crate) struct MultMenuUIElement;

// Builds plugin called MainMenuPlugin
impl Plugin for MultMenuPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_enter_system(GameState::MultiplayerMenu, setup_mult)
		.add_system_set(ConditionSet::new()
			// Only run handlers on Start state
			.run_in_state(GameState::MultiplayerMenu)
				.with_system(mult_options)
                .with_system(host_button_handler)
                .with_system(client_button_handler)
			.into())
		.add_exit_system(GameState::MultiplayerMenu, despawn_mult_menu);
	}
}

fn despawn_mult_menu(mut commands: Commands,
	button_query: Query<Entity, With<Button>>,
	camera_query: Query<Entity,  With<MenuCamera>>,
	background_query: Query<Entity, With<MultMenuBackground>>
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

fn setup_mult(mut commands: Commands,
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
		texture: asset_server.load(MULT_MENU_BACKGROUND),
		transform: Transform::from_xyz(0., 0., 0.),
		..default()
	})
	.insert(MultMenuBackground);

    // HOST BUTTON
	commands
    .spawn_bundle(ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(275.0), Val::Px(65.0)),
            // center button
            margin: UiRect::all(Val::Auto),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
        	position_type: PositionType::Absolute,
        	position: UiRect {
        	bottom: Val::Px(350.),
        	left: Val::Px((WIN_W * 0.785) / 2.),
        	..default()
        },
        ..default()
    },
        color: NORMAL_BUTTON.into(),
        ..default()
    })
    .with_children(|parent| {
        parent.spawn_bundle(TextBundle::from_section(
            "Host Game",
            TextStyle {
                font: asset_server.load("buttons/joystix monospace.ttf"),
                font_size: 40.0,
                color: TEXT_COLOR,
            },
        ));
    })
    .insert(HostButton)
    .insert(MultMenuUIElement);


// CLIENT BUTTON
commands
.spawn_bundle(ButtonBundle {
    style: Style {
        size: Size::new(Val::Px(275.0), Val::Px(65.0)),
        // center button
        margin: UiRect::all(Val::Auto),
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        position_type: PositionType::Absolute,
        position: UiRect {
            bottom: Val::Px(250.),
            left: Val::Px((WIN_W * 0.785) / 2.),
            ..default()
        },
        ..default()
    },
    color: NORMAL_BUTTON.into(),
    ..default()
})
.with_children(|parent| {
    parent.spawn_bundle(TextBundle::from_section(
        "Join Game",
        TextStyle {
            font: asset_server.load("buttons/joystix monospace.ttf"),
            font_size: 40.0,
            color: TEXT_COLOR,
        },
    ));
})
.insert(ClientButton)
.insert(MultMenuUIElement);
	
}

pub(crate) fn mult_options(mut commands: Commands, asset_server: Res<AssetServer>) 
{
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                "Select multiplayer options below.",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::TOP_CENTER)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(125.0),
                    left: Val::Px((WIN_W * 0.3) / 2.),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(MultOptionsText)
        .insert(MultMenuUIElement);
}

pub (crate) fn host_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<HostButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Host Game".to_string();
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Host Game".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Host Game".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn client_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<ClientButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Join Game".to_string();
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Join Game".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Join Game".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}