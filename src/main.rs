use battle::{create_battle_ui, load_fighters, spawn_fighters, loading_wait, extra_setup_system};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::{reflect::ReflectComponent, system::Despawn},
    prelude::*,
    reflect::{FromType, TypeRegistry, TypeRegistryInternal},
    utils::{HashMap, HashSet},
    window::PresentMode, log::{LogPlugin, LogSettings, Level},
};
use bevy_editor_pls::EditorPlugin;
use bevy_ggrs::{GGRSPlugin, Rollback, RollbackIdProvider, SessionType};
use bevy_inspector_egui::WorldInspectorPlugin;
use fighter::{
    state::{
        Active, CurrentState, Direction, Facing, HitboxData, HurtboxData, SerializedStateVec,
        StateFrame, Health, InHitstun, ProjectileReference, ProjectileData, Velocity, HBox, PlayerAxis, Animation,
    },
    systems::{
        adjust_facing_system, collision_system, hbox_position_system,
        hit_event_system, hitbox_component_system, hitbox_removal_system, hitstun_system,
        hurtbox_component_system, hurtbox_removal_system, increment_frame_system, movement_system,
        process_input_system, transition_system, ui_lifebar_system, InputBuffer, buffer_insert_system, object_system, projectile_system, axis_system, camera_system, animation_system, add_animation_player_system,
    },
    FighterPlugin,
};
use game::{
    ADD_HITBOX, ADD_HURTBOX, COLLISION, FRAME_INCREMENT, HITSTUN, HIT_EVENT, INPUT_BUFFER,
    MOVEMENT, PROCESS, REMOVE_HITBOX, REMOVE_HURTBOX, TRANSITION, UPDATE_HIT_POS, UPDATE_HURT_POS, GameState, on_round, RoundState, on_enter_loading, on_loading, on_exit_loading, on_enter_round, on_extra_setup, FACE, PROJECTILE, VELO, AXIS, 
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

use std::{env, net::SocketAddr, default};

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

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    num_players: usize,
    #[structopt(short, long)]
    check_distance: usize,
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

        .register_rollback_type::<RoundState>()

        .with_rollback_schedule(
            Schedule::default()
                .with_stage(
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
                        .with_system(
                            adjust_facing_system
                                //.run_in_state(GameState::Fight)
                                .label(FACE)
                                .after(AXIS),
                        )
                        // projectile
                        .with_system(
                            object_system
                            .after(FACE)
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
            .with_system(animation_system)
        )

        // Rollback resources
        .insert_resource(sess)
        .insert_resource(SessionType::SyncTestSession)

        // Custom Plugins
        .add_plugin(FighterPlugin)

        .register_type::<Player>()
        .insert_resource(Msaa { samples: 4 });


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
                let mut ordered: HashMap<u16, HashSet<Entity>> = HashMap::new();

                for (index, mut hitbox) in hitboxes.into_iter().enumerate() {
                    hitbox.set_id(index);

                    // let shape = shapes::Rectangle {
                    //     extents: hitbox.dimensions.truncate(),
                    //     origin: RectangleOrigin::Center,
                    // };

                    let cuboid = Cuboid::new((hitbox.dimensions / 2.).into());
                    let capsule = Capsule::new_y(hitbox.dimensions.y / 2., hitbox.dimensions.x);

                    let start_frame = hitbox.start_frame;
                    let hitbox_entity = world
                        .spawn()
                        .insert(hitbox)
                        .insert(Rollback::new(rip.next_id()))
                        .insert(Collider { shape: capsule })
                        .insert(Name::new(format!("Hitbox {}", &name)))
                        .insert(Owner(player))
                        // .insert_bundle(GeometryBuilder::build_as(
                        //     &shape,
                        //     DrawMode::Fill(FillMode::color(Color::rgba(1., 0., 0., 0.8))),
                        //     Transform::default(),
                        // ))
         
                        .insert_bundle(VisibilityBundle {
                            visibility: Visibility { is_visible: false },
                            computed: ComputedVisibility::default(),
                        })
                        .id();

                    if ordered.contains_key(&start_frame) {
                        let set = ordered.get_mut(&start_frame).unwrap();
                        set.insert(hitbox_entity);
                    } else {
                        let mut set = HashSet::<Entity>::new();
                        set.insert(hitbox_entity);
                        ordered.insert(start_frame, set);
                    }
                }
                state.add_hitboxes(ordered);
            }

            // HURTBOXES
            if let Some(hurtboxes) = hurtbox_serialized {
                let mut ordered_hurt: HashMap<u16, HashSet<Entity>> = HashMap::new();

                for hurtbox in hurtboxes {
                    // let shape = shapes::Rectangle {
                    //     extents: hurtbox.dimensions.truncate(),
                    //     origin: RectangleOrigin::Center,
                    // };

                    //let cuboid = Cuboid::new((hurtbox.dimensions / 2.).into());
                    let capsule = Capsule::new_y(hurtbox.dimensions.y / 2., hurtbox.dimensions.x);

                    let start_frame = hurtbox.start_frame.unwrap_or_default();
                    let hurtbox_entity = world
                        .spawn()
                        .insert(hurtbox)
                        .insert(Rollback::new(rip.next_id()))
                        .insert(Collider { shape: capsule })
                        .insert(Name::new(format!("Hurtbox {}", &name)))
                        .insert(Owner(player))
                        // .insert_bundle(GeometryBuilder::build_as(
                        //     &shape,
                        //     DrawMode::Fill(FillMode::color(Color::rgba(1., 1., 0., 0.8))),
                        //     Transform::default(),
                        // ))
                        .insert_bundle(VisibilityBundle {
                            visibility: Visibility { is_visible: false },
                            computed: ComputedVisibility::default(),
                        })
                        .id();

                    if ordered_hurt.contains_key(&start_frame) {
                        let set = ordered_hurt.get_mut(&start_frame).unwrap();
                        set.insert(hurtbox_entity);
                    } else {
                        let mut set = HashSet::<Entity>::new();
                        set.insert(hurtbox_entity);
                        ordered_hurt.insert(start_frame, set);
                    }
                }

                state.add_hurtboxes(ordered_hurt);
            }

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

pub fn insert_meshes(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    hitbox_query: Query<(Entity, &HitboxData)>,
    hurtbox_query: Query<(Entity, &HurtboxData)>
) {
    let hitbox_material = materials.add(Color::rgba(1., 0., 0., 0.5).into());
    let hurtbox_material = materials.add(Color::rgba(1., 1., 0., 0.5).into());


    for (entity, hitbox) in hitbox_query.iter() {
        commands.entity(entity)
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: hitbox.dimensions.x,
                    depth: hitbox.dimensions.y,
                    ..default()
                })),
                material: hitbox_material.clone(),
                visibility: Visibility { is_visible: false },
                transform: Transform::from_scale(hitbox.dimensions),
                ..default()
            });
    }

    for (entity, hurtbox) in hurtbox_query.iter() {
        commands.entity(entity)
            .insert_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: hurtbox.dimensions.x,
                    depth: hurtbox.dimensions.y,
                    ..default()
                })),
                material: hurtbox_material.clone(),
                visibility: Visibility { is_visible: false },
                transform: Transform::from_scale(hurtbox.dimensions),
                ..default()
            });
    }
}

pub fn insert_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<Entity, With<FightState>>,
    anim_player_query: Query<Entity, With<AnimationPlayer>>,
    parent_query: Query<&Parent>

) {
    for entity in query.iter() {
        commands.entity(entity)
            .insert(Animation(asset_server.load("models/ryu.glb#Animation0")));
    }

    // println!("Hey does it get here???");
    // for anim in anim_player_query.iter() {
    //     println!("What about here?");
    //     for parent in parent_query.get(anim) {
    //         for fighter in parent_query.get(parent.get()) {
    //             commands.entity(fighter.get())
    //                 .insert(AnimEntity(anim));

    //         }
    //     }

    // }

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
