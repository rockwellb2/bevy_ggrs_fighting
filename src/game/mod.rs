pub mod debug;

use std::default;

use bevy::{
    prelude::*,
    reflect::Reflect,
};

use bevy_common_assets::json::JsonAssetPlugin;

use crate::fighter::animation::components::FullBoneTransformMap;
use crate::util::pickle_asset_loader::PickleAssetPlugin;
use crate::{insert_meshes, startup, GameDebug};

use crate::{
    battle::{create_battle_ui, extra_setup_system, load_fighters, loading_wait, spawn_fighters},
    fighter::{self, systems::SetupSet, data::FighterData, systems::NonRollbackSet},
    SerializedStateVec
};

use bevy_ggrs::RollbackIdProvider;

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
    } else {
        true
    }
}

pub fn if_paused(paused: Res<Paused>) -> bool {
    if paused.0 {
        true
    } else {
        false
    }
}

pub fn on_debug(debug: Res<GameDebug>, state: Res<RoundState>) -> bool {
    if debug.0 {
        match *state {
            RoundState::Paused | RoundState::Round => true,
            _ => false,
        }
    } else {
        false
    }
}

pub fn on_debug_and_game_paused(debug: Res<GameDebug>, paused: Res<Paused>) -> bool {
    if debug.0 && paused.0 {
        true
    } else {
        false
    }
}

pub fn paused_advance_or_round(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::Paused | RoundState::AdvanceFrame | RoundState::Round => true,
        _ => false,
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

    SomethingElseEntirely,
}

pub fn on_enter_loading(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::EnterLoading => true,
        _ => false,
    }
}

pub fn on_loading(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::Loading => true,
        _ => false,
    }
}

pub fn on_exit_loading(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::ExitLoading => true,
        _ => false,
    }
}

pub fn on_round(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::Round | RoundState::AdvanceFrame => true,
        _ => false,
    }
}

pub fn on_enter_round(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::EnterRound => true,
        _ => false,
    }
}

pub fn on_extra_setup(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::ExtraSetup => true,
        _ => false,
    }
}

pub fn on_armature(state: Res<RoundState>) -> bool {
    match *state {
        RoundState::Armature => true,
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum NewGameState {
    #[default]
    SetupData,
    FighterDataLoading,
    Next,
}

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_plugins(DefaultPlugins)
            .add_plugin(JsonAssetPlugin::<SerializedStateVec>::new(&[
                "sl.json", "states",
             ]))
            .add_plugin(JsonAssetPlugin::<FighterData>::new(&["json", "fighter"]))
            .add_plugin(PickleAssetPlugin::<FullBoneTransformMap>::new(&["hurt"]))
            .add_systems(
                (load_fighters, create_battle_ui, apply_system_buffers)
                    .chain()
                    .in_set(SetupSet::Setup),
            )
            // MIGHT NEED TO APPLY THINGS BETWEEN THESE
            .add_systems(
                (loading_wait, apply_system_buffers)
                    .chain()
                    .in_set(SetupSet::Loading),
            )
            .add_systems(
                (spawn_fighters, apply_system_buffers)
                    .chain()
                    .in_set(SetupSet::ExitLoading),
            )
            .add_systems(
                (
                    fighter::animation::setup::insert_hurtbox_data,
                    fighter::animation::setup::armature_system,
                    apply_system_buffers,
                )
                    .chain()
                    .in_set(SetupSet::Armature),
            )
            .add_systems(
                (
                    startup,
                    apply_system_buffers,
                    fighter::animation::setup::insert_animations,
                    insert_meshes,
                    apply_system_buffers,
                )
                    .chain()
                    .in_set(SetupSet::EnterRound),
            )
            .add_systems(
                (
                    fighter::animation::setup::add_animation_player_system,
                    fighter::animation::setup::set_position_entity,
                    extra_setup_system,
                    apply_system_buffers,
                )
                    .chain()
                    .in_set(SetupSet::ExtraSetup),
            )
            .configure_sets((
                SetupSet::Setup.run_if(on_enter_loading),
                SetupSet::Loading.run_if(on_loading),
                SetupSet::ExitLoading.run_if(on_exit_loading),
                SetupSet::Armature.run_if(on_armature),
                SetupSet::EnterRound.run_if(on_enter_round),
                SetupSet::ExtraSetup.run_if(on_extra_setup),
                NonRollbackSet.run_if(on_round),
                // RollbackSet::Stage0
                //     .before(RollbackSet::Stage1)
                //     .run_if(on_round),
                // RollbackSet::Stage1
                //     .after(RollbackSet::Stage0)
                //     .run_if(on_round),
                // RollbackSet::Stage2
                //     .after(RollbackSet::Stage1)
                //     .run_if(on_round),
            ))
            .insert_resource(RollbackIdProvider::default())
            .insert_resource(GameDebug(false))
            
            
            ;
    }
}
