#![allow(unused)]
use crate::backgrounds::{Tile, WIN_H, WIN_W};
use crate::camera::MenuCamera;
use crate::game_client::*;
use crate::player::Player;
use crate::{battle, GameState};
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use local_ip_address::local_ip;
use rand::seq::SliceRandom;
use std::fmt::format;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};

const START_MENU_BACKGROUND: &str = "backgrounds/start_screen.png";
pub(crate) const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
pub(crate) const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub(crate) const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub(crate) const PRESSED_BUTTON: Color = Color::rgb(0.75, 0.35, 0.35);

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
        app.add_enter_system(GameState::Start, setup_menu)
            .add_system_set(
                ConditionSet::new()
                    // Only run handlers on Start state
                    .run_in_state(GameState::Start)
                    .with_system(start_button_handler)
                    .with_system(credits_button_handler)
                    .with_system(multiplayer_button_handler)
                    .with_system(help_button_handler)
                    .into(),
            )
            .add_exit_system(GameState::Start, despawn_start_menu);
    }
}

// Clears buttons from screen when ran
// Should be run after START button is pressed
fn despawn_start_menu(
    mut commands: Commands,
    button_query: Query<Entity, With<Button>>,
    camera_query: Query<Entity, With<MenuCamera>>,
    background_query: Query<Entity, With<MainMenuBackground>>,
) {
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

pub(crate) fn start_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<StartButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query
            .get_mut(*children.iter().next().unwrap())
            .unwrap();
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

pub(crate) fn credits_button_handler(
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
        let mut text = text_query
            .get_mut(*children.iter().next().unwrap())
            .unwrap();
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

pub(crate) fn multiplayer_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<MultiplayerButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
    //mut game_client: ResMut<GameClient>
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query
            .get_mut(*children.iter().next().unwrap())
            .unwrap();
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

pub(crate) fn help_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<HelpButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query
            .get_mut(*children.iter().next().unwrap())
            .unwrap();
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

fn setup_menu(
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
) {
    // -----------------------------------------------------------------------------------------------------------
    // Get an address to bind socket to
    // ::bind() creates a new socket bound to the address, and a NEW BINDING
    // CANNOT BE MADE to the same addr:port thereafter
    let socket_addresses = get_addr();
    println!("Choosing socket address from: {:?}", socket_addresses);
    let udp_socket = UdpSocket::bind(&socket_addresses[..]).unwrap();
    let socket_addr = udp_socket.local_addr().expect("Couldn't retrieve local address from socket.");
    // Set our UDP socket not to block since we need to run in frame-by-frame systems
    udp_socket.set_nonblocking(true).unwrap();
    info!("Successfully bound socket to {}", socket_addr);

    commands.insert_resource(GameClient {
        // Pass socket info over since we will need to pass the socket
        // around listener/sender systems frequently
        socket: SocketInfo {
            socket_addr,
            udp_socket,
        },
        // Default initialize player to client type
        player_type: crate::game_client::PlayerType::Client,
    });
    // -----------------------------------------------------------------------------------------------------------

    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    //creates camera for UI
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MenuCamera);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(START_MENU_BACKGROUND),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        .insert(MainMenuBackground);

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
                    bottom: Val::Px(275.),
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
                    bottom: Val::Px(200.),
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
                    bottom: Val::Px(120.),
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
