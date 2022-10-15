use bevy::{prelude::*};

pub(crate) const LEVEL_LENGTH: f32 = 5000.;
pub(crate) const LEVEL_HEIGHT: f32 = 5000.;
pub(crate) const TILE_SIZE: f32    = 16.  ;

pub(crate) const GAME_BACKGROUND: &str = "backgrounds/test_scroll_background.png";

#[derive(Component)]
pub(crate) struct Background;

