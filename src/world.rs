use {bevy::prelude::*};
use std::collections::HashMap;


#[derive(Default)]
pub(crate) struct WorldMap{
    // the first usize is the chunk id
    // the tuple is chunks relative(logical) position
    pub(crate) chunks: HashMap<usize, (isize, isize)>,
}

