use std::{io, str::from_utf8};
use crate::{backgrounds::{WIN_W}, game_client::{HostReady, HostNotReady}};
use bevy::{prelude::*};
use iyes_loopless::prelude::*;
use crate::{GameState, game_client::{GameClient, PlayerType}, monster::{MonsterStats, SelectedMonster}};
use crate::camera::MultWaitingCamera;

const MULT_WAIT_BACKGROUND: &str = "backgrounds/multiplayer_screen.png";
pub struct MultiplayerWaitingPlugin;

#[derive(Component)]
pub(crate) struct MultWaitBackground;

#[derive(Component)]
pub(crate) struct MultWaitText;

impl Plugin for MultiplayerWaitingPlugin {
	fn build(&self, app: &mut App) {
		app
		.add_enter_system(GameState::MultiplayerWaiting, setup_mult_waiting)
		.add_system_set(ConditionSet::new()
			// Only run handlers in MultiplayerWatiing state
			.run_in_state(GameState::MultiplayerWaiting)
                .with_system(mult_waiting_text)
				.with_system(host_listen_for_conn
                    .run_if(is_host)
                    .run_if_resource_exists::<HostNotReady>())
                .with_system(client_listen_for_confirmation
                    .run_if(is_client))
                .with_system(host_listen_for_confirmation
                    .run_if(is_host)
                    .run_if_resource_exists::<HostReady>())
			.into())
		.add_exit_system(GameState::MultiplayerWaiting, despawn_mult_waiting);
	}
}

fn setup_mult_waiting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, (With<Camera2d>, Without<MultWaitingCamera>)>,
) {

    cameras.for_each(|camera| {
		commands.entity(camera).despawn();
	});

    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MultWaitingCamera);

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(MULT_WAIT_BACKGROUND),
            transform: Transform::from_xyz(0., 0., 5.),
            ..default()
        })
        .insert(MultWaitBackground);


    commands.insert_resource(HostNotReady{});
}

pub(crate) fn mult_waiting_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                "Waiting for other player...",
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
        .insert(MultWaitText);
}

fn is_host(game_client: ResMut<GameClient>) -> bool {
    if game_client.player_type == PlayerType::Host {
        return true;
    }
    false
}

fn is_client(game_client: ResMut<GameClient>) -> bool {
    if game_client.player_type == PlayerType::Client {
        return true;
    }
    false
}

fn host_listen_for_conn(game_client: ResMut<GameClient>, mut commands: Commands) {
    loop {
        let mut buf = [0; 512];    
        match game_client.socket.udp_socket.recv(&mut buf) {
            Ok(result) => {                
                println!("received {result} bytes. The msg from_host_listen_for_conn is: {}", from_utf8(&buf[..result]).unwrap());
                info!("confirmation: entered host listener");
                let client_info = from_utf8(&buf[..result]).unwrap().to_string();
                info!(client_info);
                game_client.socket.udp_socket.connect(client_info).expect("Host was not able to connect to client");
                game_client.socket.udp_socket.send(b"TRUE").expect("Host was unable to send ready message to client");
                commands.remove_resource::<HostNotReady>();
                commands.insert_resource(HostReady {});
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::WouldBlock { 
                    // An ACTUAL error occurred
                    error!("{}", err);
                }
                // We're done listening
                break;
            }
        }
    }
}

fn host_listen_for_confirmation(game_client: ResMut<GameClient>, mut commands: Commands) {
    loop {
        let mut buf = [0; 512];
        match game_client.socket.udp_socket.recv(&mut buf) {
            Ok(result) => {
                info!("received {result} bytes. The msg from host_listen_for_confirmation is: {}", from_utf8(&buf[..result]).unwrap());
                info!("confirmation: entered host listen for confirmation");
                let val = from_utf8(&buf[..result]).unwrap().to_string();
                //info!("{}", val);
                if val == "TRUE" {                    
                    // Give the player a monster
                    let initial_monster_stats = MonsterStats {
                        ..Default::default()
                    };
                    commands
                        .spawn()
                        .insert_bundle(initial_monster_stats)
                        .insert(SelectedMonster);

                        commands.remove_resource::<HostReady>();
                    
                    commands.insert_resource(NextState(GameState::MultiplayerBattle));
                    // This break is necessary because the above line does not actually change state when it
                    // runs. It instead queues the state change by adding it as a resource, which will trigger
                    // state change when the system ends. The problem is, this system is set up to never end
                    // until all messages buffered have been read! Once we trigger a state change we need
                    // to leave the infinite loop to let the next buffered message be received by the appropriate
                    // new system.
                    // KEEP THIS IN MIND WHEN WRITING ANY OTHER RECEIVERS THAT CHANGE STATE
                    break;
                }
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::WouldBlock { 
                    // An ACTUAL error occurred
                    error!("{}", err);
                }
                // If it's a WouldBlock error, that just means no message has been received, so we can stop listening for this frame
                break;
            }
        }
    }
}

fn client_listen_for_confirmation(game_client: ResMut<GameClient>, mut commands: Commands) {
    loop {
        let mut buf = [0; 512];
        match game_client.socket.udp_socket.recv(&mut buf) {
            Ok(result) => {
                info!("received {result} bytes. The msg is: {}", from_utf8(&buf[..result]).unwrap());
                info!("confirmation: entered client listen for confirmation");
                let val = from_utf8(&buf[..result]).unwrap().to_string();
                info!("{}", val);
                if val == "TRUE" {
                    game_client.socket.udp_socket.send(b"TRUE").expect("Client was not able to send message to host");                    
                    // Give the player a monster in the waiting state so we can send monster info to other player in setup_mult_battle
                    let initial_monster_stats = MonsterStats {
                        ..Default::default()
                    };
                    commands
                        .spawn()
                        .insert_bundle(initial_monster_stats)
                        .insert(SelectedMonster);

                    commands.insert_resource(NextState(GameState::MultiplayerBattle));
                }
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::WouldBlock { 
                    // An ACTUAL error occurred
                    error!("{}", err);
                }
                // If it's a WouldBlock error, that just means no message has been received, so we can stop listening for this frame
                break;
            }
        }
    }
}


fn despawn_mult_waiting(
    mut commands: Commands,
    camera_query: Query<Entity, With<MultWaitingCamera>>,
    background_query: Query<Entity, With<MultWaitBackground>>,
    mult_waiting_text_query: Query<Entity, With<MultWaitText>>,
) {
    // Despawn cameras
    for c in camera_query.iter() {
        commands.entity(c).despawn();
    }
    // Despawn waiting background
    for bckg in background_query.iter() {
        commands.entity(bckg).despawn();
    }

    if mult_waiting_text_query.is_empty() {
        error!("waiting text isn't there!");
    }

    // Despawn waiting text
    mult_waiting_text_query.for_each(|mult_waiting_text| {
        commands.entity(mult_waiting_text).despawn_recursive();
    });

    //Despawn HostReady resource
    //commands.remove_resource::<HostReady>()
}