#![allow(unused)]
use std::fmt::format;
use std::net::{UdpSocket, Ipv4Addr, SocketAddr, IpAddr};
use std::sync::mpsc::{Receiver, channel, Sender};
use local_ip_address::local_ip;

use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use rand::seq::SliceRandom;
use crate::{GameState, battle};
use crate::game_client::*;
use crate::camera::{MenuCamera};
use crate::player::{Player};
use crate::backgrounds::{
	WIN_H, WIN_W, 
	Tile
};

const START_MENU_BACKGROUND: &str = "backgrounds/start_screen.png";
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

#[derive(Component)]
pub(crate) struct HelpButton; 


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
				.with_system(help_button_handler) 
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
    mut commands: Commands,
	//game_client: Res<GameClient>,
	// game_channel: Res<GameChannel>,
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Credits".to_string();
                *color = PRESSED_BUTTON.into();
                commands.insert_resource(NextState(GameState::Credits));

				// let c_sx = game_client.udp_channel.sx.clone();
    
				// // create thread for player's battle communication 
				// std::thread::spawn(move || {
				// 	let (tx, rx): (Sender<Package>, Receiver<Package>) = std::sync::mpsc::channel();

				// 	let test_pkg = Package::new(String::from("test msg from thread of player"), Some(tx.clone()));

				// 	c_sx.send(test_pkg).unwrap();

				// 	let response_from_game = rx.recv().unwrap();
				// 	println!("battle thread received confirmation here: {}", response_from_game.message);

    			// });

				// let res = game_client.udp_channel.rx.recv().unwrap();
				// let battle_thread_sx = res.sender.expect("Couldnt find sender");
				// println!("Game thread got this msg: {}", res.message);
				// let response_back = Package::new(String::from("game thread got the msg! Just confirming.."), Some(game_client.udp_channel.sx.clone()));
				// battle_thread_sx.send(response_back);

				// match game_client.udp_channel.rx.try_recv() {
				// 	Ok(pkg_response) => println!("{:?}", pkg_response.message),
				// 	Err(e) => println!("try_recv function failed: {e:?}"),
				// }
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
    mut commands: Commands,
	//mut game_client: ResMut<GameClient>
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Multiplayer".to_string();
                *color = PRESSED_BUTTON.into();
                commands.insert_resource(NextState(GameState::MultiplayerMenu));

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

pub (crate) fn help_button_handler( 
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<HelpButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
	game_client: Res<GameClient>,
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Help".to_string();
                *color = PRESSED_BUTTON.into();
                commands.insert_resource(NextState(GameState::Help));
            }
            Interaction::Hovered => {
                text.sections[0].value = "Help".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Help".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup_menu(mut commands: Commands,
	asset_server: Res<AssetServer>,
	cameras: Query<Entity, (With<Camera2d>, Without<MenuCamera>, Without<Player>, Without<Tile>)>,
){ 
// -----------------------------------------------------------------------------------------------------------
	//hardcoded for localhost for now
	let socket_addr = get_addr();
    let socket_port = socket_addr.port();
	let connection_err_msg = format!("Could not bind to {}", socket_addr);
	let udp_socket = UdpSocket::bind(socket_addr).expect(&connection_err_msg);
    let (sx, rx): (Sender<Package>, Receiver<Package>) = channel();
    let ready_for_battle = false;
	info!("Successfully binded host to {}", socket_addr);
	udp_socket.set_nonblocking(true).unwrap();
    

    commands.insert_resource(GameClient {
		socket: SocketInfo {
			socket_addr,
			udp_socket,
		},
        // send_socket: SocketInfo {
        //     socket_addr,
        //     udp_socket,
        // },
		// receive_socket: SocketInfo { socket_addr: (), udp_socket: () },
        player_type: crate::game_client::PlayerType::Client,
        udp_channel: UdpChannel {
            sx,
			rx
		},
        ready_for_battle
    });

	cameras.for_each(|camera| {
		commands.entity(camera).despawn();
	});

	//creates camera for UI
	let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MenuCamera);

	commands.spawn_bundle(SpriteBundle {
		texture: asset_server.load(START_MENU_BACKGROUND),
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
				bottom: Val::Px(190.),
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
				bottom: Val::Px(270.),
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

	// HELP BUTTON 
	commands
	.spawn_bundle(ButtonBundle {
		style: Style {
			size: Size::new(Val::Px(125.0), Val::Px(65.0)),
			// center button
			margin: UiRect::all(Val::Auto),
			// horizontally center child text
			justify_content: JustifyContent::Center,
			// vertically center child text
			align_items: AlignItems::Center,
			position_type: PositionType::Absolute,
			position: UiRect {
				bottom: Val::Px(90.),
				left: Val::Px((WIN_W * 0.900) / 2.),
				..default()
			},
			..default()
		},
		color: NORMAL_BUTTON.into(),
		..default()
	})
	.with_children(|parent| {
		parent.spawn_bundle(TextBundle::from_section(
			"Help",
			TextStyle {
				font: asset_server.load("buttons/joystix monospace.ttf"),
				font_size: 40.0,
				color: TEXT_COLOR,
			},
		));
	})
	.insert(HelpButton)
	.insert(StartMenuUIElement);
}