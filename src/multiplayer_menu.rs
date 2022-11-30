#![allow(unused)]
use crate::backgrounds::{Tile, WIN_H, WIN_W};
use crate::camera::MenuCamera;
use crate::player::Player;
use crate::GameState;
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use crate::game_client::{GameClient, self, PlayerType, Package, get_randomized_port, SocketInfo, get_addr, ClientMarker, HostMarker};
use std::fmt::format;
use std::str::from_utf8;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{io, thread};
use std::net::{UdpSocket, Ipv4Addr};
use std::sync::mpsc::{Receiver, Sender, self, channel};

const MULT_MENU_BACKGROUND: &str = "backgrounds/multiplayer_screen.png";
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
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



// Builds plugin for multiplayer menu
impl Plugin for MultMenuPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_enter_system(GameState::MultiplayerMenu, setup_mult)
        // .add_system(udp_message_listener
        //     .run_if_resource_exists::<ClientMarker>())
		.add_system_set(ConditionSet::new()
			// Only run handlers in MultiplayerMenu state
			.run_in_state(GameState::MultiplayerMenu)
				.with_system(mult_options)
                .with_system(host_button_handler)
                .with_system(client_button_handler)
			.into())
		.add_exit_system(GameState::MultiplayerMenu, despawn_mult_menu);
	}
}


// fn is_client(game_client: ResMut<GameClient>) -> bool {
//     if game_client.player_type == PlayerType::Client {
//         return true;
//     }
//     false
// }

// fn client_ready_for_battle(game_client: ResMut<GameClient>) -> bool {
//     if game_client.player_type == PlayerType::Client && game_client.ready_for_battle == true {
//         return true;
//     }
//     false
// }

/// System to listen for UDP messages. The socket is non-blocking intentionally,
/// so this works by running an "infinite" loop that will continually try to fill a 
/// 2048 byte buffer until the OS tells it that recv would block, and then it will exit the loop
/// and return.
fn udp_message_listener(game_client: ResMut<GameClient>, mut commands: Commands) {
    loop {
        let mut buf = [0; 2048];
        match game_client.socket.udp_socket.recv_from(&mut buf) {
            Ok(result) => {
                info!("Got into message checker... Read {} bytes", result.0);
                let val = String::from_utf8((&buf[0..result.0]).to_vec()).unwrap();
                info!("{}", val);
                if val == "TRUE" {
                    commands.insert_resource(NextState(GameState::MultiplayerBattle));
                }
            },
            Err(err) => {
                // If we run into this specific kind of error, it just means that the OS
                // doesn't have anything ready for us to read yet, so we will stop trying.
                // This whole system will run again on the next frame, so that's fine.
                // This error is expected to be hit a LOT, any time a message is not ready for us.
                if err.kind() != io::ErrorKind::WouldBlock { 
                    // An ACTUAL error occurred
                    error!("{}", err);
                    // This should pulse an event and then return;
                    return;
                }
                // We're done listening
                break;
            }
        }
    }
}

fn despawn_mult_menu(
    mut commands: Commands,
    camera_query: Query<Entity, With<MenuCamera>>,
    background_query: Query<Entity, With<MultMenuBackground>>,
    mult_ui_element_query: Query<Entity, With<MultMenuUIElement>>,
) {
    // Despawn cameras
    for c in camera_query.iter() {
        commands.entity(c).despawn();
    }
    // Despawn Main Menu Background
    for bckg in background_query.iter() {
        commands.entity(bckg).despawn();
    }

    if mult_ui_element_query.is_empty() {
        error!("ui elements are here!");
    }

    // Despawn multiplayer UI elements
    mult_ui_element_query.for_each(|mult_ui_element| {
        commands.entity(mult_ui_element).despawn_recursive();
    });
}

fn setup_mult(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<
        Entity,
        (
            With<Camera2d>,
            Without<MenuCamera>,
            Without<Player>,
            Without<Tile>,
        ),
    >,
    // game_channel: Res<GameChannel>,
    game_client: Res<GameClient>,
) {
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    //creates camera for UI
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MenuCamera);

    commands
        .spawn_bundle(SpriteBundle {
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


pub(crate) fn mult_options(mut commands: Commands, asset_server: Res<AssetServer>) {
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

pub(crate) fn host_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<HostButton>),
    >,
    mut text_query: Query<&mut Text>,
    // game_channel: Res<GameChannel>,
    mut game_client: ResMut<GameClient>,
    mut commands: Commands,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query
            .get_mut(*children.iter().next().unwrap())
            .unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Host Game".to_string();
                *color = PRESSED_BUTTON.into();

                // Having a listener here doesn't make sense. Networking listeners should not be attached to
                // button clicks. What this should do, most likely, is set the current game client to be a host, 
                // and change the state into a listening state. Once in a listening state, the host waits for a 
                // CONNECT or similar request, and then handles it from there. Listening should NOT occur in an 
                // interaction query, and ESPECIALLY not just one time. This should be a `loop`ed operation.
                // In all honesty, it should just be able to use the udp_message_listener system above.
                
                // If player clicks on host button, designate them as the host
                game_client.player_type = PlayerType::Host;
                commands.insert_resource(HostMarker {}); 
                commands.insert_resource(NextState(GameState::MultiplayerWaiting));

                // let mut buf = [0; 2048];
                
                // match game_client.socket.udp_socket.recv(&mut buf) {
                //     Ok(received) => {
                //         println!("received {received} bytes. The msg is: {}", from_utf8(&buf[..received]).unwrap());
                //         info!("GETS TO HOST BUTTON CLICKED");
                //         let client_info = from_utf8(&buf[..received]).unwrap().to_string();
                //         game_client.socket.udp_socket.connect(client_info);
                //         //game_client.ready_for_battle = true;
                //         // for z in 1..10 {
                //         let cloned = game_client.socket.udp_socket.try_clone().unwrap();
                //         cloned.send(b"TRUE");
                //         // }
                //         commands.insert_resource(NextState(GameState::MultiplayerBattle));
                //     },
                //     Err(e) => {
                //         //info!("No message was received: {}", e)
                //     }
                // }
                 
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

pub(crate) fn client_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<ClientButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut game_client: ResMut<GameClient>,
    mut commands: Commands,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query
            .get_mut(*children.iter().next().unwrap())
            .unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Join Game".to_string();
                *color = PRESSED_BUTTON.into();

                // if player clicks on client button, designate them as the client
                game_client.player_type = PlayerType::Client;
                commands.insert_resource(ClientMarker {}); 
                commands.insert_resource(NextState(GameState::MultiplayerWaiting));
                
                // get host IP
                println!("Enter in host IP address.");
                let mut host_ip_addr: String = String::new();
                //placeholder value for scope purposes
                let mut ipv4_addr = Ipv4Addr::new(127, 0, 0, 1);
	            match io::stdin().read_line(&mut host_ip_addr) {
		            Ok(_) => {
                         host_ip_addr = host_ip_addr.trim().to_string();
                    }
                    Err(_e) => {
                        error!("ERROR while reading in host's IP address: {}", _e);
                    }
	            }
                // get host port
                println!("Enter in host port.");
                let mut host_port: String = String::new();
	            match io::stdin().read_line(&mut host_port){
		            Ok(_) => {
                        host_port = host_port.trim().to_string();
                    }
                    Err(_e) => {
                        error!("ERROR while reading in host's port number: {}", _e);
                    }
	            }

                let host_addr_port = format!("{}:{}", host_ip_addr, host_port);
                info!("printed this: {}", host_addr_port);

                game_client.socket.udp_socket.connect(host_addr_port);
                let client_info = game_client.socket.socket_addr.to_string();
                game_client.socket.udp_socket.send(client_info.as_bytes()).expect("Error on send");
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
