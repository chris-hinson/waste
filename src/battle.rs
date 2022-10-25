#[warn(unused_imports)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use crate::{
    GameState
};
use crate::backgrounds::Tile;
use crate::camera::{MainCamera, MenuCamera, SlidesCamera};
use crate::credits::{CreditsPlugin, despawn_credits, show_slide};
use crate::player::{ANIM_TIME, AnimationTimer, Player};

const BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";


#[derive(Component)]
pub(crate) struct BattleBackground;

pub(crate) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::Battle, setup_battle)
            .add_system_set(ConditionSet::new()
                // Run these systems only when in Battle state
                .run_in_state(GameState::Battle)
                    // addl systems go here
                .into())
            .add_exit_system(GameState::Battle, despawn_battle);
    }
}

pub(crate) fn setup_battle(mut commands: Commands,
                           asset_server: Res<AssetServer>,
                           cameras: Query<Entity, (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>,
                                                   Without<Player>, Without<Tile>)>
) {
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load(BATTLE_BACKGROUND),
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    })
        .insert(BattleBackground);
}


pub(crate) fn despawn_battle(mut commands: Commands,
     camera_query: Query<Entity,  With<MainCamera>>,
     background_query: Query<Entity, With<Tile>>,
) {
    camera_query.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    background_query.for_each(|background| {
        commands.entity(background).despawn();
    });
}