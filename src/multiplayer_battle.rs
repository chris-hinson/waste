#![allow(unused)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use crate::game_client::{GameClient, self, PlayerType, Package, get_randomized_port};
use crate::{
	GameState
};
use std::str::from_utf8;
use std::{io, thread};
use std::net::{UdpSocket, Ipv4Addr};
use std::sync::mpsc::{Receiver, Sender, self};
use crate::camera::{MultCamera};
use crate::backgrounds::{
	WIN_H, WIN_W, 
	Tile
};

const MULT_BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";

#[derive(Component)]
pub(crate) struct MultBattleBackground;

pub struct MultBattlePlugin;

// Builds plugin for multiplayer battles
impl Plugin for MultBattlePlugin {
	fn build(&self, app: &mut App) {
		app
		.add_enter_system_set(GameState::MultiplayerBattle, 
            SystemSet::new()
                .with_system(setup_mult_battle)
            )
		.add_system_set(ConditionSet::new()
			// Only run handlers on Start state
			.run_in_state(GameState::MultiplayerBattle)
			.into())
		.add_exit_system(GameState::MultiplayerBattle, despawn_mult_battle);
	}
}

pub(crate) fn setup_mult_battle(mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, (With<Camera2d>, Without<MultCamera>)>,
    game_client: Res<GameClient>,
) { 

    cameras.for_each(|camera| {
		commands.entity(camera).despawn();
	});

    //creates camera for multiplayer battle background
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MultCamera);

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load(MULT_BATTLE_BACKGROUND),
        transform: Transform::from_xyz(0., 0., 10.),
        ..default()
    })  
    .insert(MultBattleBackground);
}

fn despawn_mult_battle(mut commands: Commands,
	// camera_query: Query<Entity,  With<MenuCamera>>,
	// background_query: Query<Entity, With<MultMenuBackground>>,
    // mult_ui_element_query: Query<Entity, With<MultMenuUIElement>>
){

}
