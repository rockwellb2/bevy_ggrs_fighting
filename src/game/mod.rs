pub mod debug;

use bevy::{reflect::Reflect, prelude::{Component, ResMut, Res, Resource}};

use crate::GameDebug;

pub const INPUT_BUFFER: &str = "input_buffer";
pub const HITSTUN: &str = "hitstun";
pub const FRAME_INCREMENT: &str = "frame_increment";
pub const MOD_INPUT_CHECK: &str = "modifier_input_check";
pub const PROCESS: &str = "process";
pub const TRANSITION: &str = "transition";
pub const MOVEMENT: &str = "movement";
pub const AXIS: &str = "axis";
pub const VELO: &str = "velo";
pub const FACE: &str = "face";

pub const ADD_HITBOX: &str = "insert_transform";
pub const ADD_HURTBOX: &str = "add_hurtbox";
pub const PROJECTILE: &str = "projectile";
pub const REMOVE_HITBOX: &str = "remove_hitbox";
pub const REMOVE_HURTBOX: &str = "remove_hurtbox";
pub const UPDATE_HIT_POS: &str = "update_hit_pos";
pub const UPDATE_HURT_POS: &str = "update_hurt_pos";
pub const COLLISION: &str = "collision";
pub const HIT_EVENT: &str = "hit_event";


pub const FRAME: f32 = 1. / 60.;

#[derive(Resource)]
pub struct Paused(pub bool);

pub fn not_if_paused(paused: Res<Paused>) -> bool {
    if paused.0 {
        false
    }
    else {
        true
    }

}

pub fn if_paused(paused: Res<Paused>) -> bool {
    if paused.0 {
        true
    }
    else {
        false
    }
}


pub fn on_debug(
    debug: Res<GameDebug>,
    state: Res<RoundState>
) -> bool {
    if debug.0  {
        match *state {
            RoundState::Paused | RoundState::Round => true,
            _ => false
        }
    }
    else {
        false
    }

}

pub fn on_debug_and_game_paused(
    debug: Res<GameDebug>,
    paused: Res<Paused>
) -> bool {
    if debug.0 && paused.0 {
        true
    }
    else {
        false
    }
}

pub fn paused_advance_or_round(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::Paused | RoundState::AdvanceFrame | RoundState::Round => true,
        _ => false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component, Default)]
pub enum GameState {
    #[default]
    Menu,
    LoadingFight,
    Fight,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component, Default, Resource)]
pub enum RoundState {
    #[default]
    EnterLoading,
    Loading,
    ExitLoading,
    EnterRound,
    Round,
    ExtraSetup,
    Paused,
    AdvanceFrame,
    Armature,



    SomethingElseEntirely
}


pub fn on_enter_loading(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::EnterLoading => true,
        _ => false
    }
}

pub fn on_loading(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::Loading => true,
        _ => false
    }
}

pub fn on_exit_loading(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::ExitLoading => true,
        _ => false
    }
}

pub fn on_round(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::Round | RoundState::AdvanceFrame => true,
        _ => false
    }
}

pub fn on_enter_round(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::EnterRound => true,
        _ => false
    }
}

pub fn on_extra_setup(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::ExtraSetup => true,
        _ => false
    }
}

pub fn on_armature(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::Armature => true,
        _ => false
    }
}

