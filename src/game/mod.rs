use bevy::{reflect::Reflect, prelude::{Component, ResMut, Res}, ecs::schedule::ShouldRun};

pub const INPUT_BUFFER: &str = "input_buffer";
pub const HITSTUN: &str = "hitstun";
pub const FRAME_INCREMENT: &str = "frame_increment";
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component, Default)]
pub enum GameState {
    #[default]
    Menu,
    LoadingFight,
    Fight,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Component, Default)]
pub enum RoundState {
    #[default]
    EnterLoading,
    Loading,
    ExitLoading,
    EnterRound,
    Round,
    ExtraSetup
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
   //println!("In Round!");

    match *state {
        RoundState::Round => ShouldRun::Yes,
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