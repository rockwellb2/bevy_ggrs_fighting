use battle::{create_battle_ui, load_fighters, spawn_fighters, loading_wait, extra_setup_system};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::{reflect::ReflectComponent, system::Despawn},
    prelude::*,
    reflect::{FromType, TypeRegistry, TypeRegistryInternal},
    utils::{HashMap, HashSet},
    window::PresentMode, log::{LogPlugin, LogSettings, Level}, gltf::Gltf,
};
use bevy_editor_pls::EditorPlugin;
use bevy_ggrs::{GGRSPlugin, Rollback, RollbackIdProvider, SessionType};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_scene_hook::HookPlugin;
use fighter::{
    state::{
        Active, CurrentState, Direction, Facing, HitboxData, HurtboxData, SerializedStateVec,
        StateFrame, Health, InHitstun, ProjectileReference, ProjectileData, Velocity, HBox, PlayerAxis, Animation, Hurtboxes, BoneMap, ActiveHitboxes,
    },
    systems::{
        adjust_facing_system, collision_system, hbox_position_system,
        hit_event_system, hitbox_component_system, hitbox_removal_system, hitstun_system,
        hurtbox_component_system, hurtbox_removal_system, increment_frame_system, movement_system,
        process_input_system, transition_system, ui_lifebar_system, InputBuffer, buffer_insert_system, object_system, projectile_system, axis_system, camera_system, animation_system, add_animation_player_system, pause_system, last_debug_system,
    },
    FighterPlugin, Fighter,
};
use game::{
    ADD_HITBOX, ADD_HURTBOX, COLLISION, FRAME_INCREMENT, HITSTUN, HIT_EVENT, INPUT_BUFFER,
    MOVEMENT, PROCESS, REMOVE_HITBOX, REMOVE_HURTBOX, TRANSITION, UPDATE_HIT_POS, UPDATE_HURT_POS, GameState, on_round, RoundState, on_enter_loading, on_loading, on_exit_loading, on_enter_round, on_extra_setup, FACE, PROJECTILE, VELO, AXIS, not_if_paused, Paused, if_paused, on_debug, on_debug_and_game_paused, paused_advance_or_round, on_armature, debug::state_text_system, 
};
use ggrs::{Config, PlayerType, SessionBuilder, UdpNonBlockingSocket, SyncTestSession};
//use bevy_editor_pls::prelude::*;


use bevy_common_assets::json::JsonAssetPlugin;
use bevy_prototype_lyon::prelude::*;
use input::Action;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use iyes_progress::ProgressPlugin;
use leafwing_input_manager::prelude::InputManagerPlugin;

use parry3d::{shape::{Cuboid, Capsule}};
use structopt::StructOpt;

use std::{env, net::SocketAddr, default, f32::consts::FRAC_PI_2};

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

pub struct GameDebug(pub bool);

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    num_players: usize,
    #[structopt(short, long)]
    check_distance: usize,
    #[structopt(short, long)]
    debug_mode: bool
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_BACKTRACE", "1");

    let opt = Opt::from_args();
    let num_players: usize = 2;

    let mut app = App::new();

    let mut sess_build = SessionBuilder::<GGRSConfig>::new()
        //.with_max_prediction_window(8)
        .with_check_distance(opt.check_distance)


        .with_input_delay(2)
        .with_num_players(num_players);

    
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

    for i in 0..opt.num_players {
        sess_build = sess_build.add_player(PlayerType::Local, i)?;
    }


    //let socket = UdpNonBlockingSocket::bind_to_port(opt.local_port)?;

    //let sess = sess_build.start_p2p_session(socket)?;
    let sess = sess_build.start_synctest_session()?;

    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(input::input)
        .register_rollback_type::<Transform>()
        .register_rollback_type::<CurrentState>()
        .register_rollback_type::<StateFrame>()
        .register_rollback_type::<Health>()
        .register_rollback_type::<Active>()
        .register_rollback_type::<InputBuffer>()
        .register_rollback_type::<Facing>()
        .register_rollback_type::<InHitstun>()
        .register_rollback_type::<ProjectileReference>()
        .register_rollback_type::<Velocity>()
        .register_rollback_type::<PlayerAxis>()
        .register_rollback_type::<HitboxData>()
        .register_rollback_type::<Collider>()
        .register_rollback_type::<ActiveHitboxes>()
        .register_rollback_type::<Owner>()

        .register_rollback_type::<RoundState>()

        .with_rollback_schedule(
            Schedule::default()
                .with_stage(
                    "Debug Stage 1",
                    SystemStage::parallel()
                 )
                .with_stage_after(
                    "Debug Stage 1",
                    "Setup Stage",
                    SystemStage::parallel()
                        .with_run_criteria(on_enter_loading)
                        .with_system(load_fighters.label("load_fighters"))
                        .with_system(create_battle_ui.after("load_fighters"))
                )
                .with_stage_after(
                    "Setup Stage",
                    "Loading Stage",
                    SystemStage::parallel()
                        .with_run_criteria(on_loading)
                        .with_system(loading_wait)
                )
                .with_stage_after(
                    "Loading Stage",
                    "Exit Loading Stage",
                    SystemStage::parallel()
                        .with_run_criteria(on_exit_loading)
                        .with_system(spawn_fighters)
                )
                .with_stage_after(
                    "Exit Loading Stage", 
                    "Armature Stage", 
                SystemStage::parallel()
                    .with_run_criteria(on_armature)
                    .with_system(armature_system)
            )
                .with_stage_after(
                    "Armature Stage",
                    "Enter Round Stage",
                    SystemStage::parallel()
                        .with_run_criteria(on_enter_round)
                        .with_system(startup.exclusive_system().label("startup"))
                        .with_system(insert_animations.label("insert_anim"))
                        .with_system(insert_meshes.after("insert_anim"))
                )
                .with_stage_after(
                    "Enter Round Stage",
                    "Extra Setup Stage",
                    SystemStage::parallel()
                        .with_run_criteria(on_extra_setup)
                        .with_system(add_animation_player_system.before("extra"))
                        .with_system(extra_setup_system.label("extra"))
                )
                
                .with_stage_after(
                    "Loading Stage",
                    ROLLBACK_DEFAULT,
                    SystemStage::parallel()
                        .with_run_criteria(on_round)
                        .with_system(
                            buffer_insert_system
                                .label(INPUT_BUFFER)
                                
                        )
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
                            process_input_system
                                //.run_in_state(GameState::Fight)
                                .label(PROCESS)
                                .after(FRAME_INCREMENT),
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
                        .with_system(
                            axis_system
                                .label(AXIS)
                                .after(MOVEMENT)
                        )
                        // .with_system(
                        //     adjust_facing_system
                        //         //.run_in_state(GameState::Fight)
                        //         .label(FACE)
                        //         .after(AXIS),
                        // )
                        // projectile
                        .with_system(
                            object_system
                            .label("object")
                            .after(AXIS)
                        )
                        .with_system(
                            animation_system
                            .after("object")
                        )
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
                        .with_system(
                            projectile_system
                                .after(ADD_HURTBOX)
                                .label(PROJECTILE)

                        )
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
                .with_stage_after(
                    "Collision Stage", 
                    "Debug Stage 2", 
                    SystemStage::parallel()

                ),
        )
        .build(&mut app);

    app.add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(JsonAssetPlugin::<SerializedStateVec>::new(&["sl.json"]))
        .add_plugin(JsonAssetPlugin::<FighterData>::new(&["json"]))
        
        .insert_resource(RoundState::EnterLoading)
        .add_plugin(InputManagerPlugin::<Action>::default())

        // Inspector/Editor Plugins
        .add_plugin(EditorPlugin)
        //.add_plugin(WorldInspectorPlugin::new())
        .add_plugin(FrameTimeDiagnosticsPlugin)

        // Non-rollback Systems
       .add_system_set(SystemSet::new()
            .with_run_criteria(on_round)
            .with_system(ui_lifebar_system)
            .with_system(camera_system)
            .with_system(state_text_system)
            //.with_system(animation_system)
        )

        // Debug Systems
        .add_system(bevy::window::close_on_esc)
        .add_system(pause_system.with_run_criteria(paused_advance_or_round))

        // Debug Resources
        .insert_resource(GameDebug(opt.debug_mode))

        // Rollback resources
        .insert_resource(sess)
        .insert_resource(SessionType::SyncTestSession)

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

        let access = world.get_resource::<PlayerHandleAccess>().unwrap().clone();
        let deserialized = state_lists.remove(&access.0.state_list).unwrap().0;
        let deserialized2 = deserialized.clone();

        populate_entities_with_states(world, player1, deserialized);
        populate_entities_with_states(world, player2, deserialized2);
    });


    let mut round_state = world.resource_mut::<RoundState>();
    *round_state = RoundState::ExtraSetup;
}

fn populate_entities_with_states(
    world: &mut World,
    player: Entity,
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
            let entity = world
                .spawn()
                .insert(Name::new(name.clone()))
                .insert_bundle(VisibilityBundle::default())
                .insert(Rollback::new(rip.next_id()))
                .id();


            {
                world.entity_mut(player).push_children(&[entity]);
            }

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
                        let mut set = Vec::<HitboxData>::new();
                        set.push(hitbox);
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
                let type_registry = world.get_resource::<TypeRegistry>().unwrap().clone();
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

        world.entity_mut(player)
            .insert(state_map);

    });
}

pub struct HitboxMap(pub HashMap<u32, (Handle<Mesh>, Collider)>);

pub fn insert_meshes(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,

    state_query: Query<&FightState>

) {
    let mut hitbox_resource: HashMap<u32, (Handle<Mesh>, Collider)> = HashMap::new();


    for state in state_query.iter() {
        if let Some(hitboxes) = &state.hitboxes {
            for boxes in hitboxes.values() {
                for hitbox in boxes {
                    let mesh =  meshes.add(Mesh::from(shape::Capsule {
                        radius: hitbox.radius,
                        depth: hitbox.half_height * 2. - hitbox.radius * 2.,
                        ..default()
                    }));

                    let capsule = Capsule::new_y(hitbox.half_height - hitbox.radius, hitbox.radius);
                    let collider: Collider = capsule.into();
                    hitbox_resource.insert(hitbox.global_id.expect("GlobalID doesn't exist"), (mesh, collider));
                }
            }
        }
    }

    commands.insert_resource(HitboxMap(hitbox_resource));
}

pub fn insert_animations(
    mut commands: Commands,
    assets_gltf: Res<Assets<Gltf>>,
    handle_access: Res<PlayerHandleAccess>,
    query: Query<(Entity, &Name)>,
    fighter_query: Query<(&Player, &StateMap), With<Fighter>>,

    animation_clips: Res<Assets<AnimationClip>>

    //mut armature_query: Query<(&Name, &mut Transform)>
) {
    for (player, map) in fighter_query.iter() {
        let handle = handle_access.get(player.0).model.clone();
        let gltf = assets_gltf.get(&handle).expect("GLTF handle doesn't exist");
        let animations = &gltf.named_animations;
        for (entity, name) in query.iter_many(map.map.values()) {
            if let Some(animation) = animations.get(&name.to_string()) {
                let length = animation_clips
                    .get(&animation)
                    .expect("AnimationClip doesn't exist")
                    .duration();

                commands.entity(entity)
                    .insert(Animation(animation.clone(), length));
            }
        }
    }
}

#[derive(Component)]
pub struct AnimEntity(pub Entity);

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}

#[derive(Default, Reflect, Hash, Component)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}


pub fn armature_system(
    mut commands: Commands,
    hurtbox_query: Query<(Entity, &HurtboxData)>,
    parent_query: Query<&Parent>,
    mut state: ResMut<RoundState>,
    mut fighter_query: Query<&mut Hurtboxes>
) {

    let hurt_iter = hurtbox_query.iter();

    if hurt_iter.len() > 0 {
        for (hurt_ent, _hurtbox) in hurt_iter {
            let mut ancestor = parent_query.get(hurt_ent).expect("Entity doesn't have Parent");
            loop {
                if let Ok(parent) = parent_query.get(ancestor.get()) {
                    ancestor = parent;
                }
                else {
                    break;
                }
            }

            commands.entity(hurt_ent)
                .insert(Owner(ancestor.get()));
            
            let mut hurtboxes = fighter_query.get_mut(ancestor.get()).expect("Does not have Hurtboxes component");
            hurtboxes.add(hurt_ent);


        }

        *state = RoundState::EnterRound;
    }

    
}

