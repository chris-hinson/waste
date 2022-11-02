#[warn(unused_imports)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use local_ip_address::local_ip;
use crate::{
	GameState
};
use std::net::{UdpSocket, SocketAddr, Ipv4Addr, IpAddr};
use std::sync::mpsc::{Sender, self};
use std::sync::mpsc::Receiver;
use std::{thread, io};
use crate::camera::{MenuCamera};
use crate::player::{Player, PlayerType, Socket, GameClient};
use crate::backgrounds::{
	WIN_H, WIN_W, 
	Tile
};

const MULT_MENU_BACKGROUND: &str = "backgrounds/multiplayer_screen.png";
const TEXT_COLOR: Color = Color::rgb(0.9,0.9,0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.75, 0.35, 0.35);
const MSG_SIZE: usize = 50;


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
	camera_query: Query<Entity,  With<MenuCamera>>,
	background_query: Query<Entity, With<MultMenuBackground>>,
    mult_ui_element_query: Query<Entity, With<MultMenuUIElement>>
){
	// Despawn cameras
	for c in camera_query.iter() {
		commands.entity(c).despawn();
	}
	// Despawn Main Menu Background
	for bckg in background_query.iter() {
		commands.entity(bckg).despawn();
	}

    if mult_ui_element_query.is_empty() 
    {
    error!("ui elements are here!");
    }

    // Despawn multiplayer UI elements
    mult_ui_element_query.for_each(|mult_ui_element| {
        commands.entity(mult_ui_element).despawn_recursive();
   });
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


    // // generating player UDP socket
    // let player_addr = get_addr();
    // let socket = UdpSocket::bind(player_addr).unwrap();

    // //commands.insert_resource(Player);

    // // add the player's socket info (IP, port number, UDP socket)
    // commands.insert_resource(PlayerSocket { 
    //     addr: player_addr, 
    //     socket: socket 
    // });

    // set the player's type to undefined for now (change if/when they select Multiplayer)
    // commands.insert_resource(PlayerType {
    //     player_type: SocketType::Unidentified
    // });


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
    mut game_client_query: Query<&mut GameClient>,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    if game_client_query.is_empty() {
        info!("no matches for host!");
        return;
    }
    let mut game_client = game_client_query.get_single_mut().unwrap();

    // Change player struct element `player_type` to be SocketType::Host
    // So you have to query player as mutable to modify their elements
    // commands.entity(player).insert(PlayerType {player_type: SocketType::Host});

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Host Game".to_string();
                *color = PRESSED_BUTTON.into();

                game_client.player_type = PlayerType::Host;

                //Changing GameState to PreHost
                commands.insert_resource(NextState(GameState::PreHost));
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

    let mut host_info: String = String::new();
	match io::stdin().read_line(&mut host_info){
		Ok(_) => {
			host_info = host_info.trim().to_string();
		}
		Err(e) => {
            info!("Error connecting to host {}", e);
		}
	}

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Join Game".to_string();
                *color = PRESSED_BUTTON.into();

                //Channels to communicate with client thread
                // let (sx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
                // let (stx, rrx): (Sender<String>, Receiver<String>) = mpsc::channel();

                // //Declaring separate thread to manage client loop{}
                // let udp_conn_client = create_client();
                // thread::spawn(move || {
                //     client(udp_conn_client, sx, rrx);
                // });
                

                //Set GameState to Peer
                // commands.insert_resource(&udp_conn_client);
                commands.insert_resource(NextState(GameState::PrePeer));
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


pub(crate) fn host(socket: UdpSocket, tx: Sender<String>, erx: Receiver<String>) -> std::io::Result<()>
{
    println!("Host active:");

    loop {
        let mut buf = [0 as u8; MSG_SIZE];
        let (num_bytes, src_addr) = socket.recv_from(&mut buf)?;
        let msg = String::from_utf8((&buf[0..num_bytes]).to_vec()).unwrap();


        //send client data through channel
        match tx.send(String::from(msg)){
            Ok(_) => {
                //no issues sending msg
            }
            Err(e) => {
                println!("Error sending message: {}", e)
            }
        }


        //see if there are any messages to send a client
        match erx.try_recv() {
            Ok(msg) => {
                let e = String::from("Error: ".to_owned() + &msg);
                socket.send_to(e.as_bytes(), src_addr);
            }
            Err(e) => {
                //no message in queue, do nothing
            }
        }

    }
}

pub(crate) fn client(socket: UdpSocket, sx: Sender<String>, rrx: Receiver<String>) -> std::io::Result<()> {
    // let mut buf = [0; 10];
    // let (number_of_bytes, src_addr) = udp_conn_client.recv_from(&mut buf)
    //     .expect("Didn't receive data");
    // let filled_buf = &mut buf[..number_of_bytes];
    // println!("{}, {}", number_of_bytes, src_addr);

    println!("Client UDP Connected");

    loop {
        let mut buf = [0 as u8; MSG_SIZE];
        let (num_bytes, src_addr) = socket.recv_from(&mut buf)?;
        let msg = String::from_utf8((&buf[0..num_bytes]).to_vec()).unwrap();
        println!("{}, {}", msg, src_addr);


        //Send msg through channel back to calling fn
        match sx.send(String::from(msg)){
            Ok(_) => { }
            Err(e) => {
                println!("Error sending msg: {}", e)
            }
        }

        //see if there are any error messages 
        match rrx.try_recv() {
            Ok(msg) => {
                let e = String::from("Error: ".to_owned() + &msg);
                socket.send_to(e.as_bytes(), src_addr).expect("TODO: panic message");
            }
            Err(e) => {
                
            }
        }

    }
}
