use bevy::{prelude::*};
//use bevy_inspector_egui::RegisterInspectable;




use self::{
    data::{FighterData, HitEvent}, 
    state::{StateMap, CurrentState, State as FightState, StateFrame, Facing, Health, Direction, ProjectileReference, Exclude, Velocity}, 
    systems::InputBuffer, event::TransitionEvent,
    modifiers::{Movement, InputTransition, AdjustFacing, CreateObject, Velo, OnExitSetPos, InputWindowCheck, InputMet, OnExitZeroVelo}, hit::components::HitboxData
};

pub(crate) mod state;
pub(crate) mod systems;
pub(crate) mod data;
pub(crate) mod event;
pub(crate) mod modifiers;
pub(crate) mod animation;
pub(crate) mod hit;

#[derive(Component)]
pub struct Fighter;


pub struct FighterPlugin;

impl Plugin for FighterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<HitEvent>()
            .add_event::<TransitionEvent>()

            // Modifiers
            .register_type::<Movement>()
            .register_type::<InputTransition>()
            .register_type::<AdjustFacing>()
            .register_type::<CreateObject>()
            .register_type::<Velo>()
            .register_type::<OnExitSetPos>()
            .register_type::<OnExitZeroVelo>()
            .register_type::<InputWindowCheck>()


            .register_type::<Facing>()
            .register_type::<Health>()
            .register_type::<InputBuffer>()
            .register_type::<ProjectileReference>()
            .register_type::<Exclude>()

            // These registers below are purely for the inspector
            .register_type::<CurrentState>()
            .register_type::<StateFrame>()
            .register_type::<FightState>()
            .register_type::<HitboxData>()
            .register_type::<StateMap>()
            .register_type::<Direction>()
            .register_type::<InputMet>()
            .register_type::<Velocity>()

        
            .register_type::<ProjectileReference>()



            
            .register_type::<FighterData>();
    }
}