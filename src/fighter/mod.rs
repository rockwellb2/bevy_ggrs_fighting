use bevy::{prelude::*};
use bevy_inspector_egui::RegisterInspectable;




use self::{data::{FighterData, HitEvent}, state::{StateMap, CurrentState, State as FightState, HitboxData, StateFrame, Movement, InputTransition, AdjustFacing, Facing, Health, Direction}, systems::InputBuffer, event::TransitionEvent};

pub(crate) mod state;
pub(crate) mod systems;
pub(crate) mod data;
pub(crate) mod event;

#[derive(Component)]
pub struct Fighter;


pub struct FighterPlugin;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<HitEvent>()
            .add_event::<TransitionEvent>()
            .register_type::<Movement>()
            .register_type::<InputTransition>()
            .register_type::<AdjustFacing>()
            .register_type::<Facing>()
            .register_type::<Health>()
            .register_type::<InputBuffer>()

            // These registers below are purely for the inspector
            .register_type::<CurrentState>()
            .register_type::<StateFrame>()
            .register_type::<FightState>()
            .register_type::<HitboxData>()
            .register_type::<StateMap>()
            .register_type::<Direction>()
            .register_inspectable::<Direction>()
            .register_type::<FighterData>();
    }
}