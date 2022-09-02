use battle::{load_fighters, spawn_fighters, create_battle_ui};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::reflect::ReflectComponent,
    prelude::*,
    reflect::{FromType, TypeRegistry, TypeRegistryInternal},
    utils::{HashMap, HashSet},
    window::PresentMode,
};
use bevy_editor_pls::EditorPlugin;
use bevy_ggrs::{GGRSPlugin, Rollback, SessionType};
use fighter::{
    state::{
        CurrentState, SerializedStateVec,
        Variables, Active, Direction, Facing, HitboxData, HurtboxData,
    },
    systems::{
        buffer_insert_system, hitbox_component_system, increment_frame_system, movement_system,
        process_input_system, InputBuffer, hitbox_removal_system, adjust_facing_system, hurtbox_component_system, hurtbox_removal_system, hbox_position_system, collision_system, hit_event_system, ui_lifebar_system, hitstun_system, transition_system,
    },
    FighterPlugin,
};
use game::{INPUT_BUFFER, PROCESS, MOVEMENT, ADD_HURTBOX, ADD_HITBOX, REMOVE_HITBOX, REMOVE_HURTBOX, UPDATE_HIT_POS, UPDATE_HURT_POS, COLLISION, HIT_EVENT, HITSTUN, FRAME_INCREMENT, TRANSITION};
use ggrs::{Config, PlayerType, SessionBuilder};
//use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use bevy_common_assets::json::JsonAssetPlugin;
use input::Action;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use iyes_progress::ProgressPlugin;
use bevy_prototype_lyon::prelude::*;
use leafwing_input_manager::prelude::InputManagerPlugin;
use nalgebra::{Vector, Vector3};
use parry3d::{shape::Cuboid, math::Real};

use std::{

    env,

};

use crate::{
    battle::{PlayerEntities, PlayerHandleAccess},
    fighter::{
        data::{FighterData, Collider},
        state::{
            SerializedState, State as FightState,
            StateMap, Owner,
        },
  
    }
};

mod battle;
mod fighter;
mod input;
mod util;
mod game;

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
    type Address = String;
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct Player(u8);

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let mut app = App::new();

    let mut sess_build = SessionBuilder::<GGRSConfig>::new().with_num_players(2);

    sess_build = sess_build
        .add_player(PlayerType::Local, 1)
        .expect("Try something");

    let sess = sess_build
        .start_synctest_session()
        .expect("Session can't be built");

    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(input::input)
        .register_rollback_type::<Transform>()
        .register_rollback_type::<CurrentState>()
        .register_rollback_type::<Variables>()
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
                                .after(INPUT_BUFFER)
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
                                .after(PROCESS)
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
                                .label(ADD_HITBOX)
                        )
                        .with_system(
                            hurtbox_component_system
                                .run_in_state(GameState::Fight)
                                .after(ADD_HITBOX)
                                .label(ADD_HURTBOX)
                        )
                        .with_system(
                            hitbox_removal_system
                                .run_in_state(GameState::Fight)
                                .label(REMOVE_HITBOX)
                                .after(ADD_HURTBOX)
                        )
                        .with_system(
                            hurtbox_removal_system
                                .run_in_state(GameState::Fight)
                                .label(REMOVE_HURTBOX)
                                .after(REMOVE_HITBOX)
                        )
                        .with_system(
                            hbox_position_system::<HitboxData>
                                .run_in_state(GameState::Fight)
                                .label(UPDATE_HIT_POS)
                                .after(REMOVE_HURTBOX)
                        )
                        .with_system(
                            hbox_position_system::<HurtboxData>
                                .run_in_state(GameState::Fight)
                                .label(UPDATE_HURT_POS)
                                .after(UPDATE_HIT_POS)
                        )
                )
                .with_stage_after(
                    "Hitbox Stage",
                    "Collision Stage",
                    SystemStage::parallel()
                        .with_system(
                            collision_system
                                .run_in_state(GameState::Fight)
                                .label(COLLISION)
                        )
                        .with_system(
                            hit_event_system
                                .run_in_state(GameState::Fight)
                                .label(HIT_EVENT)
                                .after(COLLISION)
                        )
                        // .with_system(
                        //     ui_lifebar_system
                        //         .run_in_state(GameState::Fight)
                        //         .after(HIT_EVENT)

                        // )
                    )
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

        .add_system(ui_lifebar_system.run_in_state(GameState::Fight))

        .add_plugin(InputManagerPlugin::<Action>::default())

        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(EditorPlugin)
        //.add_plugin(WorldInspectorPlugin::new())
        .insert_resource(sess)
        .insert_resource(SessionType::SyncTestSession)
        .add_plugin(FighterPlugin)
        .register_type::<Player>()
        .insert_resource(Msaa { samples: 4 });

    app.run();
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
                let hitbox_entity = 
                    world
                        .spawn()
                        .insert(hitbox)
                        .insert(Collider {
                            shape: cuboid
                        })
                        .insert(Name::new(format!("Hitbox {}", &name)))
                        .insert(Owner(player))
                        .insert_bundle(GeometryBuilder::build_as(
                            &shape,
                            DrawMode::Fill(FillMode::color(Color::rgba(1., 0., 0., 0.8))),
                            Transform::default()
                        ))
                        .insert_bundle(VisibilityBundle {
                            visibility: Visibility { is_visible: false },
                            computed: ComputedVisibility::default()
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
                let hurtbox_entity = 
                    world
                        .spawn()
                        .insert(hurtbox)
                        .insert(Collider {
                            shape: cuboid
                        })
                        .insert(Name::new(format!("Hurtbox {}", &name)))
                        .insert(Owner(player))
                        .insert_bundle(GeometryBuilder::build_as(
                            &shape,
                            DrawMode::Fill(FillMode::color(Color::rgba(1., 1., 0., 0.8))),
                            Transform::default()
                        ))
                        .insert_bundle(VisibilityBundle {
                            visibility: Visibility { is_visible: false },
                            computed: ComputedVisibility::default()
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
            target.transitions.insert(*state_map.get(&t).unwrap());
        }
    }

    world.entity_mut(player).insert(state_map);
}


pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}

#[derive(Default, Reflect, Hash, Component)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}
