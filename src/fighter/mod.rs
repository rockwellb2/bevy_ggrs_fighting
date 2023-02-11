use bevy::{prelude::*};




use crate::game::RoundState;

use self::{
    data::{FighterData, HitEvent, Collider, SegmentProxy}, 
    state::{StateMap, CurrentState, State as FightState, HitboxData, StateFrame, Facing, Health, Direction, ProjectileData, ProjectileReference, PlayerAxis, Animation, HurtboxData, Exclude, Owner, Active, Velocity, InHitstun}, 
    systems::InputBuffer, event::TransitionEvent,
    modifiers::{Movement, InputTransition, AdjustFacing, CreateObject, Object, Velo, VectorType, OnExitSetPos, InputWindowCheck}
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
            .register_type::<OnExitSetPos>()
            .register_type::<InputWindowCheck>()


            .register_type::<Facing>()
            .register_type::<Health>()
            .register_type::<InputBuffer>()
            .register_type::<ProjectileReference>()
            .register_type::<Exclude>()



            // I might need these
            .register_type::<Owner>()
            .register_type::<Active>()
            .register_type::<Velocity>()
            .register_type::<InHitstun>()
            .register_type::<RoundState>()

            // These registers below are purely for the inspector
            .register_type::<CurrentState>()
            .register_type::<StateFrame>()
            .register_type::<FightState>()
            .register_type::<HitboxData>()
            .register_type::<StateMap>()
            .register_type::<Direction>()
            .register_type::<ProjectileData>()
            .register_type::<Object>()
            .register_type::<VectorType>()
            .register_type::<Option<VectorType>>()
            .register_type::<PlayerAxis>()
            .register_type::<HurtboxData>()
            .register_type::<Collider>()
            .register_type::<SegmentProxy>()

            .register_type::<ProjectileReference>()
            .register_type::<FighterData>();
    }
}