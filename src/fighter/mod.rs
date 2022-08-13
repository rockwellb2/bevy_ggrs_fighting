use bevy::prelude::*;
use bevy_inspector_egui::RegisterInspectable;




use self::{data::FighterData, state::{StateMap, CurrentState, State as FightState, HitboxData, StateFrame, Movement, InputTransition, Owner, AdjustFacing, Facing}, systems::InputBuffer};

pub(crate) mod state;
pub(crate) mod systems;
pub(crate) mod data;

#[derive(Component)]
pub struct Fighter;


pub struct FighterPlugin;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Movement>()
            .register_type::<InputTransition>()
            .register_type::<AdjustFacing>()
            .register_type::<Facing>()
            // Inspectable
            .register_inspectable::<Owner>()
            .register_inspectable::<Direction>()
            //.register_inspectable::<FightState>

            // These registers below are purely for the inspector
            .register_type::<CurrentState>()
            .register_type::<StateFrame>()
            .register_type::<FightState>()
            .register_type::<HitboxData>()
            .register_type::<InputBuffer>()
            .register_type::<StateMap>()
            .register_type::<Direction>()
            .register_type::<FighterData>();
    }
}