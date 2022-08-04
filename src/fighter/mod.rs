use bevy::prelude::*;
use ggrs::InputStatus;

use crate::input::{Input as FightInput, LEFT, RIGHT};

use self::data::FighterData;

pub(crate) mod state;
pub(crate) mod systems;
pub(crate) mod data;

#[derive(Component)]
pub struct Fighter;


pub struct FighterPlugin;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<FighterData>();
    }
}