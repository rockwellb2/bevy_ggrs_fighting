use battle::{create_battle_ui, load_fighters, spawn_fighters};
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
use fighter::{
    state::{
        Active, CurrentState, Direction, Facing, HitboxData, HurtboxData, SerializedStateVec,
        Variables, StateFrame, Health,
    },
    systems::{
        adjust_facing_system, buffer_insert_system, collision_system, hbox_position_system,
        hit_event_system, hitbox_component_system, hitbox_removal_system, hitstun_system,
        hurtbox_component_system, hurtbox_removal_system, increment_frame_system, movement_system,
        process_input_system, transition_system, ui_lifebar_system, InputBuffer,
    },
    FighterPlugin,
};
use game::{
    ADD_HITBOX, ADD_HURTBOX, COLLISION, FRAME_INCREMENT, HITSTUN, HIT_EVENT, INPUT_BUFFER,
    MOVEMENT, PROCESS, REMOVE_HITBOX, REMOVE_HURTBOX, TRANSITION, UPDATE_HIT_POS, UPDATE_HURT_POS,
};
use ggrs::{Config, PlayerType, SessionBuilder, UdpNonBlockingSocket, SyncTestSession};
//use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use bevy_common_assets::json::JsonAssetPlugin;
use bevy_prototype_lyon::prelude::*;
use input::Action;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use iyes_progress::ProgressPlugin;
use leafwing_input_manager::prelude::InputManagerPlugin;
use nalgebra::{Vector, Vector3};
use parry3d::{math::Real, shape::Cuboid};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    Menu,
    LoadingFight,
    Fight,
}

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
        //.register_rollback_type::<Variables>()
        .register_rollback_type::<Active>()
        .register_rollback_type::<InputBuffer>()
        .register_rollback_type::<Facing>()
        //.register_rollback_type::<FrameCount>()
        .with_rollback_schedule(
            Schedule::default()
                .with_stage(
                    ROLLBACK_DEFAULT,
                    SystemStage::parallel()
                        .with_system(
                            buffer_insert_system
                                .run_in_state(GameState::Fight)
                                .label(INPUT_BUFFER),
                        )
                        .with_system(
                            hitstun_system
                                .run_in_state(GameState::Fight)
                                .label(HITSTUN)
                                .after(INPUT_BUFFER),
                        )
                        .with_system(
                            increment_frame_system
                                .run_in_state(GameState::Fight)
                                .label(FRAME_INCREMENT)
                                .after(HITSTUN),
                        )
                        .with_system(
                            process_input_system
                                .run_in_state(GameState::Fight)
                                .label(PROCESS)
                                .after(FRAME_INCREMENT),
                        )
                        .with_system(
                            transition_system
                                .run_in_state(GameState::Fight)
                                .label(TRANSITION)
                                .after(PROCESS),
                        )
                        .with_system(
                            movement_system
                                .run_in_state(GameState::Fight)
                                .label(MOVEMENT)
                                .after(TRANSITION),
                        )
                        .with_system(
                            adjust_facing_system
                                .run_in_state(GameState::Fight)
                                .after(MOVEMENT),
                        ),
                )
                .with_stage_after(
                    ROLLBACK_DEFAULT,
                    "Hitbox Stage",
                    SystemStage::parallel()
                        .with_system(
                            hitbox_component_system
                                .run_in_state(GameState::Fight)
                                .label(ADD_HITBOX),
                        )
                        .with_system(
                            hurtbox_component_system
                                .run_in_state(GameState::Fight)
                                .after(ADD_HITBOX)
                                .label(ADD_HURTBOX),
                        )
                        .with_system(
                            hitbox_removal_system
                                .run_in_state(GameState::Fight)
                                .label(REMOVE_HITBOX)
                                .after(ADD_HURTBOX),
                        )
                        .with_system(
                            hurtbox_removal_system
                                .run_in_state(GameState::Fight)
                                .label(REMOVE_HURTBOX)
                                .after(REMOVE_HITBOX),
                        )
                        .with_system(
                            hbox_position_system::<HitboxData>
                                .run_in_state(GameState::Fight)
                                .label(UPDATE_HIT_POS)
                                .after(REMOVE_HURTBOX),
                        )
                        .with_system(
                            hbox_position_system::<HurtboxData>
                                .run_in_state(GameState::Fight)
                                .label(UPDATE_HURT_POS)
                                .after(UPDATE_HIT_POS),
                        ),
                )
                .with_stage_after(
                    "Hitbox Stage",
                    "Collision Stage",
                    SystemStage::parallel()
                        .with_system(
                            collision_system
                                .run_in_state(GameState::Fight)
                                .label(COLLISION),
                        )
                        .with_system(
                            hit_event_system
                                .run_in_state(GameState::Fight)
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
        // Game State Systems and Related Plugins
        .add_loopless_state(GameState::LoadingFight)
        .add_plugin(
            ProgressPlugin::new(GameState::LoadingFight)
                .continue_to(GameState::Fight)
                .track_assets(),
        )
        .add_enter_system(GameState::LoadingFight, load_fighters)
        .add_enter_system(GameState::LoadingFight, create_battle_ui)
        .add_exit_system(GameState::LoadingFight, spawn_fighters)
        .add_enter_system(GameState::Fight, startup.exclusive_system())
        //.add_system(ui_lifebar_system.run_in_state(GameState::Fight))
        .add_plugin(InputManagerPlugin::<Action>::default())

        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EditorPlugin)


        //.add_plugin(LogPlugin)
        // .add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(WorldInspectorPlugin::new())
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
        })



        .insert_resource(sess)

        //.insert_resource(SessionType::P2PSession)
        .insert_resource(SessionType::SyncTestSession)
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
            println!("Name: {}", name);
            let entity = world
                .spawn()
                .insert(Name::new(name.clone()))
                .insert_bundle(VisibilityBundle::default())
                //.insert(Rollback::new(rip.next_id()))
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

            println!("State {}: {:?}", name, state.triggers);

            // HITBOXES
            if let Some(hitboxes) = hbox_serialized {
                let mut ordered: HashMap<u16, HashSet<Entity>> = HashMap::new();

                for hitbox in hitboxes {
                    let shape = shapes::Rectangle {
                        extents: hitbox.dimensions.truncate(),
                        origin: RectangleOrigin::Center,
                    };

                    let cuboid = Cuboid::new((hitbox.dimensions / 2.).into());

                    let start_frame = hitbox.start_frame;
                    let hitbox_entity = world
                        .spawn()
                        .insert(hitbox)
                        //.insert(Rollback::new(rip.next_id()))
                        .insert(Collider { shape: cuboid })
                        .insert(Name::new(format!("Hitbox {}", &name)))
                        .insert(Owner(player))
                        .insert_bundle(GeometryBuilder::build_as(
                            &shape,
                            DrawMode::Fill(FillMode::color(Color::rgba(1., 0., 0., 0.8))),
                            Transform::default(),
                        ))
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
                println!("Hitboxes: {:?}", ordered);
                state.add_hitboxes(ordered);
            }

            // HURTBOXES
            if let Some(hurtboxes) = hurtbox_serialized {
                let mut ordered_hurt: HashMap<u16, HashSet<Entity>> = HashMap::new();

                for hurtbox in hurtboxes {
                    let shape = shapes::Rectangle {
                        extents: hurtbox.dimensions.truncate(),
                        origin: RectangleOrigin::Center,
                    };

                    let cuboid = Cuboid::new((hurtbox.dimensions / 2.).into());

                    let start_frame = hurtbox.start_frame.unwrap_or_default();
                    let hurtbox_entity = world
                        .spawn()
                        .insert(hurtbox)
                        //.insert(Rollback::new(rip.next_id()))
                        .insert(Collider { shape: cuboid })
                        .insert(Name::new(format!("Hurtbox {}", &name)))
                        .insert(Owner(player))
                        .insert_bundle(GeometryBuilder::build_as(
                            &shape,
                            DrawMode::Fill(FillMode::color(Color::rgba(1., 1., 0., 0.8))),
                            Transform::default(),
                        ))
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

        println!("StateMap: {:?}", state_map);

        for (s, transitions) in transition_list {
            let mut target = world.get_mut::<FightState>(s).unwrap();
            println!("State {} Transitions: {:?}", target.id, transitions);
            for t in transitions {
                target.transitions.push(*state_map.get(&t).unwrap());
            }
        }

        world.entity_mut(player)
            .insert(state_map)
            .insert(Rollback::new(rip.next_id()))
            ;

    });
}

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}

#[derive(Default, Reflect, Hash, Component)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}
