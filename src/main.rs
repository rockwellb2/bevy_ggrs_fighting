use battle::{create_battle_ui, extra_setup_system, load_fighters, loading_wait, spawn_fighters};
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, ecs::reflect::ReflectComponent, prelude::*,
    reflect::TypeRegistry, utils::HashMap,
};
use bevy_editor_pls::EditorPlugin;
use bevy_ggrs::{GGRSPlugin, Rollback, RollbackIdProvider, Session};

use bevy_scene_hook::HookPlugin;
use fighter::{
    state::{
        Active, ActiveHitboxes, CurrentState, Facing, HBox, Health, HitboxData,
        HurtboxData, InHitstun, PlayerAxis, ProjectileReference,
        SerializedStateVec, StateFrame, Velocity,
    },
    systems::{
        axis_system, buffer_insert_system, camera_system, collision_system,
        hbox_position_system, hit_event_system, hitbox_component_system, hitbox_removal_system,
        hitstun_system, hurtbox_component_system, hurtbox_removal_system, increment_frame_system, modifier_input_check, movement_system, object_system, pause_system,
        process_input_system, projectile_system, transition_system, ui_lifebar_system, InputBuffer,
    },
    Fighter, FighterPlugin,
};
use game::{
    debug::state_text_system, not_if_paused, on_armature, on_enter_loading, on_enter_round, on_exit_loading, on_extra_setup,
    on_loading, on_round, paused_advance_or_round, Paused, RoundState, ADD_HITBOX,
    ADD_HURTBOX, AXIS, COLLISION, FRAME_INCREMENT, HITSTUN, HIT_EVENT, INPUT_BUFFER,
    MOD_INPUT_CHECK, MOVEMENT, PROCESS, PROJECTILE, REMOVE_HITBOX, REMOVE_HURTBOX, TRANSITION,
    UPDATE_HIT_POS, UPDATE_HURT_POS,
};
use ggrs::Config;
//use bevy_editor_pls::prelude::*;

use bevy_common_assets::json::JsonAssetPlugin;
use bevy_prototype_lyon::prelude::*;
use input::Action;


use leafwing_input_manager::prelude::InputManagerPlugin;

use parry3d::shape::{Capsule, Cuboid};
use structopt::StructOpt;

use std::{env, net::SocketAddr};




use crate::{
    battle::{PlayerEntities, PlayerHandleAccess},
    fighter::{
        data::{Collider, FighterData},
        state::{Owner, SerializedState, State as FightState, StateMap},
    },
};

mod battle;
mod fighter;
mod game;
mod input;
mod util;

const FPS: usize = 60;
const ROLLBACK_DEFAULT: &str = "rollback_default";

#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = input::Input;
    type State = u8;
    type Address = SocketAddr;
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct Player(u8);

// #[derive(StructOpt)]
// struct Opt {
//     #[structopt(short, long)]
//     local_port: u16,
//     #[structopt(short, long)]
//     players: Vec<String>,
//     #[structopt(short, long)]
//     spectators: Vec<SocketAddr>

// }

#[derive(Resource)]
pub struct GameDebug(pub bool);

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
        .with_input_system(input::input)
        .register_rollback_component::<Transform>()
        .register_rollback_component::<CurrentState>()
        .register_rollback_component::<StateFrame>()
        .register_rollback_component::<Health>()
        .register_rollback_component::<Active>()
        .register_rollback_component::<InputBuffer>()
        .register_rollback_component::<Facing>()
        .register_rollback_component::<InHitstun>()
        .register_rollback_component::<ProjectileReference>()
        .register_rollback_component::<Velocity>()
        .register_rollback_component::<PlayerAxis>()
        .register_rollback_component::<HitboxData>()
        .register_rollback_component::<Collider>()
        .register_rollback_component::<ActiveHitboxes>()
        .register_rollback_component::<Owner>()
        .register_rollback_resource::<RoundState>()
        .with_rollback_schedule(
            Schedule::default()
                
                .with_stage(
                    ROLLBACK_DEFAULT,
                    SystemStage::parallel()
                        .with_run_criteria(on_round)
                        .with_system(buffer_insert_system.label(INPUT_BUFFER))
                        .with_system(
                            hitstun_system
                                //.run_in_state(GameState::Fight)
                                .label(HITSTUN)
                                .after(INPUT_BUFFER),
                        )
                        .with_system(
                            increment_frame_system
                                //.run_in_state(GameState::Fight)
                                .with_run_criteria(not_if_paused)
                                .label(FRAME_INCREMENT)
                                .after(HITSTUN),
                        )
                        .with_system(
                            modifier_input_check
                                .label(MOD_INPUT_CHECK)
                                .after(FRAME_INCREMENT),
                        )
                        .with_system(
                            process_input_system
                                //.run_in_state(GameState::Fight)
                                .label(PROCESS)
                                .after(MOD_INPUT_CHECK),
                        )
                        .with_system(
                            transition_system
                                //.run_in_state(GameState::Fight)
                                .label(TRANSITION)
                                .after(PROCESS),
                        )
                        .with_system(
                            movement_system
                                //.run_in_state(GameState::Fight)
                                .label(MOVEMENT)
                                .after(TRANSITION),
                        )
                        .with_system(axis_system.label(AXIS).after(MOVEMENT))
                        // .with_system(
                        //     adjust_facing_system
                        //         //.run_in_state(GameState::Fight)
                        //         .label(FACE)
                        //         .after(AXIS),
                        // )
                        // projectile
                        .with_system(object_system.label("object").after(AXIS))
                        .with_system(
                            fighter::animation::rollback::animation_system.after("object"),
                        ),
                )
                .with_stage_after(
                    ROLLBACK_DEFAULT,
                    "Hitbox Stage",
                    SystemStage::parallel()
                        .with_run_criteria(on_round)
                        .with_system(
                            hitbox_component_system
                                //.run_in_state(GameState::Fight)
                                .label(ADD_HITBOX),
                        )
                        .with_system(
                            hurtbox_component_system
                                //.run_in_state(GameState::Fight)
                                .after(ADD_HITBOX)
                                .label(ADD_HURTBOX),
                        )
                        .with_system(projectile_system.after(ADD_HURTBOX).label(PROJECTILE))
                        .with_system(
                            hitbox_removal_system
                                //.run_in_state(GameState::Fight)
                                .label(REMOVE_HITBOX)
                                .after(PROJECTILE),
                        )
                        .with_system(
                            hurtbox_removal_system
                                //.run_in_state(GameState::Fight)
                                .label(REMOVE_HURTBOX)
                                .after(REMOVE_HITBOX),
                        )
                        .with_system(
                            hbox_position_system::<HitboxData>
                                //.run_in_state(GameState::Fight)
                                .label(UPDATE_HIT_POS)
                                .after(REMOVE_HURTBOX),
                        )
                        .with_system(
                            hbox_position_system::<HurtboxData>
                                //.run_in_state(GameState::Fight)
                                .label(UPDATE_HURT_POS)
                                .after(UPDATE_HIT_POS),
                        ),
                )
                .with_stage_after(
                    "Hitbox Stage",
                    "Collision Stage",
                    SystemStage::parallel()
                        .with_run_criteria(on_round)
                        .with_system(
                            collision_system
                                //.run_in_state(GameState::Fight)
                                .label(COLLISION),
                        )
                        .with_system(
                            hit_event_system
                                //.run_in_state(GameState::Fight)
                                .label(HIT_EVENT)
                                .after(COLLISION),
                        ),
                )
                .with_stage_after("Collision Stage", "Debug Stage 2", SystemStage::parallel()),
        )
        .build(&mut app);

    app.add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(JsonAssetPlugin::<SerializedStateVec>::new(&[
            "sl.json", "states",
        ]))
        .add_plugin(JsonAssetPlugin::<FighterData>::new(&["json", "data"]))
        .insert_resource(RoundState::EnterLoading)
        .add_plugin(InputManagerPlugin::<Action>::default())
        // Inspector/Editor Plugins
        .add_plugin(EditorPlugin)
        //.add_plugin(WorldInspectorPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin)
        // Non-rollback Systems
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(on_round)
                .with_system(ui_lifebar_system)
                .with_system(camera_system)
                .with_system(state_text_system), //.with_system(animation_system)
        )
        .add_stage(
            "Setup Stage",
            SystemStage::parallel()
                .with_run_criteria(on_enter_loading)
                .with_system(load_fighters.label("load_fighters"))
                .with_system(create_battle_ui.after("load_fighters")),
        )
        .add_stage_after(
            "Setup Stage",
            "Loading Stage",
            SystemStage::parallel()
                .with_run_criteria(on_loading)
                .with_system(loading_wait),
        )
        .add_stage_after(
            "Loading Stage",
            "Exit Loading Stage",
            SystemStage::parallel()
                .with_run_criteria(on_exit_loading)
                .with_system(spawn_fighters),
        )
        .add_stage_after(
            "Exit Loading Stage",
            "Armature Stage",
            SystemStage::parallel()
                .with_run_criteria(on_armature)
                .with_system(fighter::animation::setup::insert_hurtbox_data.label("insert_hdata"))
                .with_system(fighter::animation::setup::armature_system.after("insert_hdata")),
        )
        .add_stage_after(
            "Armature Stage",
            "Enter Round Stage",
            SystemStage::parallel()
                .with_run_criteria(on_enter_round)
                .with_system(startup.label("startup"))
                .with_system(
                    fighter::animation::setup::insert_animations
                        .label("insert_anim")
                )
                .with_system(insert_meshes.after("insert_anim")),
        )
        .add_stage_after(
            "Enter Round Stage",
            "Extra Setup Stage",
            SystemStage::parallel()
                .with_run_criteria(on_extra_setup)
                .with_system(fighter::animation::setup::add_animation_player_system.before("extra"))
                .with_system(extra_setup_system.label("extra")),
        )
        // Debug Systems
        .add_system(bevy::window::close_on_esc)
        .add_system(pause_system.with_run_criteria(paused_advance_or_round))
        // Debug Resources
        .insert_resource(GameDebug(opt.debug_mode))
        // Rollback resources
        // Custom Plugins
        .add_plugin(FighterPlugin)
        .add_plugin(HookPlugin)
        .register_type::<Player>()
        .insert_resource(Paused(false))
        .insert_resource(Msaa { samples: 1 });

    app.run();

    Ok(())
}

fn startup(world: &mut World) {
    world.resource_scope(|world, mut state_lists: Mut<Assets<SerializedStateVec>>| {
        let players = world.get_resource::<PlayerEntities>().unwrap();
        let player1 = players.get(1);
        let player2 = players.get(2);

        //let access = world.get_resource::<PlayerHandleAccess>().unwrap().clone();
        let access = <&battle::PlayerHandleAccess>::clone(
            &world.get_resource::<PlayerHandleAccess>().unwrap(),
        );

        let deserialized = &state_lists.get(&access.0.state_list).unwrap().0;
        let deserialized2 = deserialized.clone();

        populate_entities_with_states(world, player1, 1, deserialized.to_vec());
        populate_entities_with_states(world, player2, 2, deserialized2);
    });

    let mut round_state = world.resource_mut::<RoundState>();
    *round_state = RoundState::ExtraSetup;
}

fn populate_entities_with_states(
    world: &mut World,
    player: Entity,
    player_num: u8,
    deserialized: Vec<SerializedState>,
) {
    let mut state_map = StateMap::new();
    let mut transition_list: Vec<(Entity, Vec<u16>)> = Vec::new();

    let mut global_hitbox_id: u32 = 0;

    world.resource_scope(|world, mut rip: Mut<RollbackIdProvider>| {
        for mut state in deserialized {
            let name = state
                .debug_name
                .as_ref()
                .unwrap_or(&"State".to_string())
                .clone();

            let name = format!("({}) {}", player_num, name);
            let entity = world
                .spawn(())
                .insert(Name::new(name.clone()))
                .insert(VisibilityBundle::default())
                .insert(Rollback::new(rip.next_id()))
                .insert(Owner(player))
                .id();

            //world.entity_mut(player).push_children(&[entity]);

            state_map.add_state(state.id, entity);
            let hbox_serialized = state.unsorted_hitboxes.take();
            let hurtbox_serialized = state.unsorted_hurtboxes.take();
            let mods_serialized = state.modifiers.take();
            let transitions_serialized = state.transitions.clone();

            transition_list.push((entity, transitions_serialized));

            let mut state = FightState::from_serialized(state);

            // HITBOXES
            if let Some(hitboxes) = hbox_serialized {
                let mut ordered: HashMap<u16, Vec<HitboxData>> = HashMap::new();

                for (index, mut hitbox) in hitboxes.into_iter().enumerate() {
                    hitbox.set_id(index);
                    hitbox.set_global_id(global_hitbox_id);
                    global_hitbox_id += 1;

                    //let capsule = Capsule::new_y(hitbox.half_height - hitbox.radius, hitbox.radius);

                    let start_frame = hitbox.start_frame;

                    if ordered.contains_key(&start_frame) {
                        let set = ordered.get_mut(&start_frame).unwrap();
                        set.push(hitbox);
                    } else {
                        let set = vec![hitbox];
                        ordered.insert(start_frame, set);
                    }
                }
                state.add_hitboxes(ordered);
            }

            // HURTBOXES
            // if let Some(hurtboxes) = hurtbox_serialized {
            //     let mut ordered_hurt: HashMap<u16, HashSet<Entity>> = HashMap::new();

            //     for hurtbox in hurtboxes {
            //         // let shape = shapes::Rectangle {
            //         //     extents: hurtbox.dimensions.truncate(),
            //         //     origin: RectangleOrigin::Center,
            //         // };

            //         //let cuboid = Cuboid::new((hurtbox.dimensions / 2.).into());
            //         let capsule = Capsule::new_y(hurtbox.half_height, hurtbox.radius);

            //         let start_frame = hurtbox.start_frame.unwrap_or_default();
            //         let hurtbox_entity = world
            //             .spawn()
            //             .insert(hurtbox)
            //             .insert(Rollback::new(rip.next_id()))
            //             .insert(Collider { shape: capsule })
            //             .insert(Name::new(format!("Hurtbox {}", &name)))
            //             .insert(Owner(player))
            //             // .insert_bundle(GeometryBuilder::build_as(
            //             //     &shape,
            //             //     DrawMode::Fill(FillMode::color(Color::rgba(1., 1., 0., 0.8))),
            //             //     Transform::default(),
            //             // ))
            //             .insert_bundle(VisibilityBundle {
            //                 visibility: Visibility { is_visible: false },
            //                 computed: ComputedVisibility::default(),
            //             })
            //             .id();

            //         if ordered_hurt.contains_key(&start_frame) {
            //             let set = ordered_hurt.get_mut(&start_frame).unwrap();
            //             set.insert(hurtbox_entity);
            //         } else {
            //             let mut set = HashSet::<Entity>::new();
            //             set.insert(hurtbox_entity);
            //             ordered_hurt.insert(start_frame, set);
            //         }
            //     }

            //     state.add_hurtboxes(ordered_hurt);
            // }

            // MODIFIERS
            if let Some(modifiers) = mods_serialized {
                let type_registry = world.get_resource::<AppTypeRegistry>().unwrap().clone();
                let type_registry = type_registry.read();

                for modifier in modifiers {
                    let modifier = modifier.clone_value();
                    let registration = type_registry.get_with_name(modifier.type_name()).unwrap();

                    let reflect_component = registration.data::<ReflectComponent>().unwrap();
                    reflect_component.insert(world, entity, &**&modifier);
                }
            }

            world.entity_mut(entity).insert(state);
        }

        for (s, transitions) in transition_list {
            let mut target = world.get_mut::<FightState>(s).unwrap();
            for t in transitions {
                target.transitions.push(*state_map.get(&t).unwrap());
            }
        }

        world.entity_mut(player).insert(state_map);
    });
}

#[derive(Resource)]
pub struct HitboxMap(pub HashMap<u32, (Handle<Mesh>, Collider)>);

pub fn insert_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    state_query: Query<&FightState>,
) {
    let mut hitbox_resource: HashMap<u32, (Handle<Mesh>, Collider)> = HashMap::new();

    for state in state_query.iter() {
        if let Some(hitboxes) = &state.hitboxes {
            for boxes in hitboxes.values() {
                for hitbox in boxes {
                    let mesh = meshes.add(Mesh::from(shape::Capsule {
                        radius: hitbox.radius,
                        depth: hitbox.half_height * 2. - hitbox.radius * 2.,
                        ..default()
                    }));

                    let capsule = Capsule::new_y(hitbox.half_height - hitbox.radius, hitbox.radius);
                    let collider: Collider = capsule.into();
                    hitbox_resource.insert(
                        hitbox.global_id.expect("GlobalID doesn't exist"),
                        (mesh, collider),
                    );
                }
            }
        }
    }

    commands.insert_resource(HitboxMap(hitbox_resource));
}

#[derive(Component)]
pub struct AnimEntity(pub Entity);

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}

#[derive(Default, Reflect, Hash, Component, Resource)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}
