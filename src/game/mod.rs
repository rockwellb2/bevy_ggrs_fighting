pub mod debug;

use bevy::{reflect::Reflect, prelude::{Component, ResMut, Res, Resource}, ecs::schedule::ShouldRun};

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

pub fn not_if_paused(paused: Res<Paused>) -> ShouldRun {
    return if paused.0 {
        ShouldRun::No
    }
    else {
        ShouldRun::Yes
    }

}

pub fn if_paused(paused: Res<Paused>) -> ShouldRun {
    return if paused.0 {
        ShouldRun::Yes
    }
    else {
        ShouldRun::No
    }
}


pub fn on_debug(
    debug: Res<GameDebug>,
    state: Res<RoundState>
) -> ShouldRun {
    if debug.0  {
        match *state {
            RoundState::Paused | RoundState::Round => ShouldRun::Yes,
            _ => ShouldRun::No
        }
    }
    else {
        ShouldRun::No
    }

}

pub fn on_debug_and_game_paused(
    debug: Res<GameDebug>,
    paused: Res<Paused>
) -> ShouldRun {
    if debug.0 && paused.0 {
        ShouldRun::Yes
    }
    else {
        ShouldRun::No
    }
}

pub fn paused_advance_or_round(state: Res<RoundState>) -> ShouldRun {
    match *state {
        RoundState::Paused | RoundState::AdvanceFrame | RoundState::Round => ShouldRun::Yes,
        _ => ShouldRun::No
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component, Default)]
pub enum GameState {
    #[default]
    Menu,
    LoadingFight,
    Fight,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component, Resource, Default)]
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
    Armature
}


pub fn on_enter_loading(state: Res<RoundState>) -> ShouldRun {
    match *state {
        RoundState::EnterLoading => ShouldRun::Yes,
        _ => ShouldRun::No
    }
}

pub fn on_loading(state: Res<RoundState>) -> ShouldRun {
    match *state {
        RoundState::Loading => ShouldRun::Yes,
        _ => ShouldRun::No
    }
}

pub fn on_exit_loading(state: Res<RoundState>) -> ShouldRun {
    match *state {
        RoundState::ExitLoading => ShouldRun::Yes,
        _ => ShouldRun::No
    }
}

pub fn on_round(state: Res<RoundState>) -> ShouldRun {
    match *state {
        RoundState::Round | RoundState::AdvanceFrame => ShouldRun::Yes,
        _ => ShouldRun::No
    }
}

pub fn on_enter_round(state: Res<RoundState>) -> ShouldRun {
    match *state {
        RoundState::EnterRound => ShouldRun::Yes,
        _ => ShouldRun::No
    }
}

pub fn on_extra_setup(state: Res<RoundState>) -> ShouldRun {
    match *state {
        RoundState::ExtraSetup => ShouldRun::Yes,
        _ => ShouldRun::No
    }
}

pub fn on_armature(state: Res<RoundState>) -> ShouldRun {
    match *state {
        RoundState::Armature => ShouldRun::Yes,
        _ => ShouldRun::No
    }
}

