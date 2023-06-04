use bevy_fighting_lib::battle::{create_battle_ui, extra_setup_system, load_fighters, loading_wait, spawn_fighters};
use bevy_fighting_lib::{GGRSConfig, FPS, GameDebug, Player};
use bevy_fighting_lib::fighter;


use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, ecs::reflect::ReflectComponent, prelude::*,
    reflect::TypeRegistry, utils::HashMap,
};
use bevy_editor_pls::EditorPlugin;
use bevy_ggrs::{GGRSPlugin, GGRSSchedule, Rollback, RollbackIdProvider, Session};

use bevy_fighting_lib::fighter::{
    hit::components::{AirborneHitstun, HitboxData},
    state::{
        Active, ActiveHitboxes, CurrentState, Facing, GroundedHitstun, HBox, Health, HurtboxData,
        PlayerAxis, ProjectileReference, SerializedStateVec, StateFrame, Velocity,
    },
    systems::{
        axis_system, buffer_insert_system, camera_system, collision_system, hbox_position_system,
        hit_event_system, hitbox_component_system, hitbox_removal_system, hitstun_system,
        hurtbox_component_system, hurtbox_removal_system, increment_frame_system,
        modifier_input_check, movement_system, object_system, pause_system, process_input_system,
        projectile_system, transition_system, ui_lifebar_system, InputBuffer, NonRollbackSet,
        RollbackSet, SetupSet,
    },
    Fighter, FighterPlugin,
};
use bevy_fighting_lib::game::{
    debug::state_text_system, not_if_paused, on_armature, on_enter_loading, on_enter_round,
    on_exit_loading, on_extra_setup, on_loading, on_round, paused_advance_or_round, Paused,
    RoundState, ADD_HITBOX, ADD_HURTBOX, AXIS, COLLISION, FRAME_INCREMENT, HITSTUN, HIT_EVENT,
    INPUT_BUFFER, MOD_INPUT_CHECK, MOVEMENT, PROCESS, PROJECTILE, REMOVE_HITBOX, REMOVE_HURTBOX,
    TRANSITION, UPDATE_HIT_POS, UPDATE_HURT_POS, SetupPlugin,
};
use ggrs::Config;
//use bevy_editor_pls::prelude::*;

use bevy_common_assets::json::JsonAssetPlugin;
use bevy_fighting_lib::input::Action;

use leafwing_input_manager::prelude::InputManagerPlugin;

use parry3d::shape::{Capsule, Cuboid};
use structopt::StructOpt;

use std::{env, net::SocketAddr};

use bevy_fighting_lib::{
    battle::{PlayerEntities, PlayerHandleAccess},
    fighter::{
        data::{Collider, FighterData},
        state::{Owner, SerializedState, State as FightState, StateMap},
    },
};

// mod battle;
// mod fighter;
// mod game;
// mod input;
// mod util;





// #[derive(StructOpt)]
// struct Opt {
//     #[structopt(short, long)]
//     local_port: u16,
//     #[structopt(short, long)]
//     players: Vec<String>,
//     #[structopt(short, long)]
//     spectators: Vec<SocketAddr>

// }

// #[derive(Resource)]
// pub struct GameDebug(pub bool);

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    num_players: usize,
    #[structopt(short, long)]
    check_distance: usize,
    #[structopt(short, long)]
    debug_mode: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_BACKTRACE", "1");

    // let config = aws_config::from_env().region(gamelift::Region::new("us-east-1")).load().await;
    // let client = cognito::Client::new(&config);
    // let x = client
    //     .get_id()
    //     .set_identity_pool_id(Some("us-east-1:a049d9ef-a70c-428c-a06b-7b95b6409934".into()))
    //     .send().await;

    // match x {
    //     Ok(_a) => {

    //     },
    //     Err(b) => {
    //         match b {
    //             cognito::types::SdkError::ConstructionFailure(_) => todo!(),
    //             cognito::types::SdkError::TimeoutError(_) => todo!(),
    //             cognito::types::SdkError::DispatchFailure(_) => todo!(),
    //             cognito::types::SdkError::ResponseError { err, raw } => todo!(),
    //             cognito::types::SdkError::ServiceError { err, raw } => todo!(),
    //         }

    //     }

    // }

    let opt = Opt::from_args();
    let num_players: usize = 2;

    let mut app = App::new();

    // let mut sess_build = SessionBuilder::<GGRSConfig>::new()
    //     //.with_max_prediction_window(8)
    //     .with_check_distance(opt.check_distance)
    //     .with_input_delay(2)
    //     .with_num_players(num_players);

    // for (i, player_addr) in opt.players.iter().enumerate() {
    //     if player_addr == "localhost" {
    //         // local players
    //         sess_build = sess_build.add_player(PlayerType::Local, i)?;
    //     } else {
    //         // remote players
    //         let remote_addr: SocketAddr = player_addr.parse()?;
    //         sess_build = sess_build.add_player(PlayerType::Remote(remote_addr), i)?;
    //     }
    // }

    // for i in 0..opt.num_players {
    //     sess_build = sess_build.add_player(PlayerType::Local, i)?;
    // }

    //let socket = UdpNonBlockingSocket::bind_to_port(opt.local_port)?;

    //let sess = sess_build.start_p2p_session(socket)?;
    //let sess = sess_build.start_synctest_session()?;

    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(bevy_fighting_lib::input::input)
        .register_rollback_component::<Transform>()
        .register_rollback_component::<CurrentState>()
        .register_rollback_component::<StateFrame>()
        .register_rollback_component::<Health>()
        .register_rollback_component::<Active>()
        .register_rollback_component::<InputBuffer>()
        .register_rollback_component::<Facing>()
        .register_rollback_component::<GroundedHitstun>()
        .register_rollback_component::<AirborneHitstun>()
        .register_rollback_component::<ProjectileReference>()
        .register_rollback_component::<Velocity>()
        .register_rollback_component::<PlayerAxis>()
        .register_rollback_component::<HitboxData>()
        .register_rollback_component::<Collider>()
        .register_rollback_component::<ActiveHitboxes>()
        .register_rollback_component::<Owner>()
        .register_rollback_resource::<RoundState>()
        // .with_rollback_schedule(
        //     Schedule::default()
        //         .with_stage(
        //             ROLLBACK_DEFAULT,
        //             SystemStage::parallel()
        //                 .with_run_criteria(on_round)
        //                 .with_system(buffer_insert_system.label(INPUT_BUFFER))
        //                 .with_system(
        //                     hitstun_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(HITSTUN)
        //                         .after(INPUT_BUFFER),
        //                 )
        //                 .with_system(
        //                     increment_frame_system
        //                         //.run_in_state(GameState::Fight)
        //                         .with_run_criteria(not_if_paused)
        //                         .label(FRAME_INCREMENT)
        //                         .after(HITSTUN),
        //                 )
        //                 .with_system(
        //                     modifier_input_check
        //                         .label(MOD_INPUT_CHECK)
        //                         .after(FRAME_INCREMENT),
        //                 )
        //                 .with_system(
        //                     process_input_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(PROCESS)
        //                         .after(MOD_INPUT_CHECK),
        //                 )
        //                 .with_system(
        //                     transition_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(TRANSITION)
        //                         .after(PROCESS),
        //                 )
        //                 .with_system(
        //                     movement_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(MOVEMENT)
        //                         .after(TRANSITION),
        //                 )
        //                 .with_system(axis_system.label(AXIS).after(MOVEMENT))
        //                 // .with_system(
        //                 //     adjust_facing_system
        //                 //         //.run_in_state(GameState::Fight)
        //                 //         .label(FACE)
        //                 //         .after(AXIS),
        //                 // )
        //                 // projectile
        //                 .with_system(object_system.label("object").after(AXIS))
        //                 .with_system(
        //                     fighter::animation::rollback::animation_system.after("object"),
        //                 ),
        //         )
        //         .with_stage_after(
        //             ROLLBACK_DEFAULT,
        //             "Hitbox Stage",
        //             SystemStage::parallel()
        //                 .with_run_criteria(on_round)
        //                 .with_system(
        //                     hitbox_component_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(ADD_HITBOX),
        //                 )
        //                 .with_system(
        //                     hurtbox_component_system
        //                         //.run_in_state(GameState::Fight)
        //                         .after(ADD_HITBOX)
        //                         .label(ADD_HURTBOX),
        //                 )
        //                 .with_system(projectile_system.after(ADD_HURTBOX).label(PROJECTILE))
        //                 .with_system(
        //                     hitbox_removal_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(REMOVE_HITBOX)
        //                         .after(PROJECTILE),
        //                 )
        //                 .with_system(
        //                     hurtbox_removal_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(REMOVE_HURTBOX)
        //                         .after(REMOVE_HITBOX),
        //                 )
        //                 .with_system(
        //                     hbox_position_system::<HitboxData>
        //                         //.run_in_state(GameState::Fight)
        //                         .label(UPDATE_HIT_POS)
        //                         .after(REMOVE_HURTBOX),
        //                 )
        //                 .with_system(
        //                     hbox_position_system::<HurtboxData>
        //                         //.run_in_state(GameState::Fight)
        //                         .label(UPDATE_HURT_POS)
        //                         .after(UPDATE_HIT_POS),
        //                 ),
        //         )
        //         .with_stage_after(
        //             "Hitbox Stage",
        //             "Collision Stage",
        //             SystemStage::parallel()
        //                 .with_run_criteria(on_round)
        //                 .with_system(
        //                     collision_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(COLLISION),
        //                 )
        //                 .with_system(
        //                     hit_event_system
        //                         //.run_in_state(GameState::Fight)
        //                         .label(HIT_EVENT)
        //                         .after(COLLISION),
        //                 ),
        //         )
        //         .with_stage_after("Collision Stage", "Debug Stage 2", SystemStage::parallel()),
        // )
        .build(&mut app);

    app
        .add_plugin(SetupPlugin)
        //.add_plugin(ShapePlugin)
        .insert_resource(RoundState::EnterLoading)
        .add_plugin(InputManagerPlugin::<Action>::default())
        // Inspector/Editor Plugins
        .add_plugin(EditorPlugin::default())
        //.add_plugin(WorldInspectorPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .edit_schedule(GGRSSchedule, |schedule| {
            schedule.configure_sets((
                SetupSet::Setup.run_if(on_enter_loading),
                SetupSet::Armature.run_if(on_armature),
                SetupSet::EnterRound.run_if(on_enter_round),
                SetupSet::ExtraSetup.run_if(on_extra_setup),
                NonRollbackSet.run_if(on_round),
                RollbackSet::Stage0
                    .before(RollbackSet::Stage1)
                    .run_if(on_round),
                RollbackSet::Stage1
                    .after(RollbackSet::Stage0)
                    .run_if(on_round),
                RollbackSet::Stage2
                    .after(RollbackSet::Stage1)
                    .run_if(on_round),
            ));
        })
        .add_systems(
            (
                apply_system_buffers,
                buffer_insert_system,
                hitstun_system,
                increment_frame_system,
                modifier_input_check,
                process_input_system,
                transition_system,
                movement_system,
                axis_system,
                object_system,
                fighter::animation::rollback::animation_system,
                apply_system_buffers,
            )
                .chain()
                .in_set(RollbackSet::Stage0)
                .in_schedule(GGRSSchedule),
        )
        .add_systems(
            (
                hitbox_component_system,
                hurtbox_component_system,
                hitbox_removal_system,
                hurtbox_removal_system,
                projectile_system,
                hbox_position_system::<HitboxData>,
                hbox_position_system::<HurtboxData>,
                apply_system_buffers,
                collision_system,
                hit_event_system,
                apply_system_buffers
            )
                .chain()
                .in_set(RollbackSet::Stage1)
                .in_schedule(GGRSSchedule),
        )
        // Non-rollback Systems
        .add_systems((ui_lifebar_system, camera_system, state_text_system).in_set(NonRollbackSet))
        // Non-rollback Systems
        // .add_system_set(
        //     SystemSet::new()
        //         .with_run_criteria(on_round)
        //         .with_system(ui_lifebar_system)
        //         .with_system(camera_system)
        //         .with_system(state_text_system), //.with_system(animation_system)
        // )
        // .add_stage(
        //     "Setup Stage",
        //     SystemStage::parallel()
        //         .with_run_criteria(on_enter_loading)
        //         .with_system(load_fighters.label("load_fighters"))
        //         .with_system(create_battle_ui.after("load_fighters")),
        // )
        // .add_stage_after(
        //     "Setup Stage",
        //     "Loading Stage",
        //     SystemStage::parallel()
        //         .with_run_criteria(on_loading)
        //         .with_system(loading_wait),
        // )
        // .add_stage_after(
        //     "Loading Stage",
        //     "Exit Loading Stage",
        //     SystemStage::parallel()
        //         .with_run_criteria(on_exit_loading)
        //         .with_system(spawn_fighters),
        // )
        // .add_stage_after(
        //     "Exit Loading Stage",
        //     "Armature Stage",
        //     SystemStage::parallel()
        //         .with_run_criteria(on_armature)
        //         .with_system(fighter::animation::setup::insert_hurtbox_data.label("insert_hdata"))
        //         .with_system(fighter::animation::setup::armature_system.after("insert_hdata")),
        // )
        // .add_stage_after(
        //     "Armature Stage",
        //     "Enter Round Stage",
        //     SystemStage::parallel()
        //         .with_run_criteria(on_enter_round)
        //         .with_system(startup.label("startup"))
        //         .with_system(fighter::animation::setup::insert_animations.label("insert_anim"))
        //         .with_system(insert_meshes.after("insert_anim")),
        // )
        // .add_stage_after(
        //     "Enter Round Stage",
        //     "Extra Setup Stage",
        //     SystemStage::parallel()
        //         .with_run_criteria(on_extra_setup)
        //         .with_system(fighter::animation::setup::add_animation_player_system.before("extra"))
        //         .with_system(extra_setup_system.label("extra")),
        // )
        // Debug Systems
        .add_system(bevy::window::close_on_esc)
        .add_system(pause_system.run_if(paused_advance_or_round))
        // Debug Resources
        .insert_resource(GameDebug(opt.debug_mode))
        // Rollback resources
        // Custom Plugins
        .add_plugin(FighterPlugin)
        //.add_plugin(HookPlugin)
        .register_type::<Player>()
        .insert_resource(Paused(false))
        .insert_resource(Msaa::Sample2);

    app.run();

    Ok(())
}
