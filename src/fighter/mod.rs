use bevy::prelude::*;
use ggrs::InputStatus;
use ruwren::Class;

use crate::input::{Input as FightInput, LEFT, RIGHT};

pub(crate) mod state;

#[derive(Component)]
pub struct Fighter;


