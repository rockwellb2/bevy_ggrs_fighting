use bevy::{prelude::*};
use bevy_inspector_egui::RegisterInspectable;




use self::{
    data::{FighterData, HitEvent}, 
    state::{StateMap, CurrentState, State as FightState, HitboxData, StateFrame, Facing, Health, Direction, ProjectileData, ProjectileReference, PlayerAxis, Animation, HurtboxData}, 
    systems::InputBuffer, event::TransitionEvent,
    modifiers::{Movement, InputTransition, AdjustFacing, CreateObject, Object, Velo, VectorType}
};

pub(crate) mod state;
pub(crate) mod systems;
pub(crate) mod data;
pub(crate) mod event;
pub(crate) mod modifiers;

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


            .register_type::<Facing>()
            .register_type::<Health>()
            .register_type::<InputBuffer>()
            .register_type::<ProjectileReference>()

            // These registers below are purely for the inspector
            .register_type::<CurrentState>()
            .register_type::<StateFrame>()
            .register_type::<FightState>()
            .register_type::<HitboxData>()
            .register_type::<StateMap>()
            .register_type::<Direction>()
            .register_inspectable::<Direction>()
            .register_inspectable::<HitboxData>()
            .register_inspectable::<ProjectileData>()
            .register_inspectable::<Object>()
            .register_inspectable::<VectorType>()
            .register_inspectable::<Option<VectorType>>()
            .register_inspectable::<PlayerAxis>()
            .register_inspectable::<HurtboxData>()
            //.register_inspectable::<ProjectileReference>()
            .register_type::<ProjectileReference>()
            .register_type::<FighterData>();
    }
}