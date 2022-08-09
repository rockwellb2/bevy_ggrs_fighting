use battle::{load_fighters, spawn_fighters};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::reflect::ReflectComponent,
    prelude::*,
    reflect::{FromType, TypeRegistry, TypeRegistryInternal},
    utils::{HashMap, HashSet},
    window::PresentMode,
};
use bevy_ggrs::{GGRSPlugin, Rollback, SessionType};
use fighter::{
    state::{
        CurrentState, HitboxData, InputTransition, Movement, SerializedStateVec, StateModifier,
        Variables,
    },
    systems::{
        buffer_insert_system, hitbox_component_system, increment_frame_system, movement_system,
        process_input_system, something_system, InputBuffer,
    },
    FighterPlugin,
};
use ggrs::{Config, PlayerType, SessionBuilder};
//use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use bevy_common_assets::json::JsonAssetPlugin;
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem};
use iyes_progress::ProgressPlugin;

use std::{
    any::Any,
    env,
    mem::{self, size_of},
};

use crate::{
    battle::{PlayerEntities, PlayerHandleAccess},
    fighter::{
        data::FighterData,
        state::{
            self, state_system, ReflectStateModifier, SerializedState, State as FightState,
            StateFrame, StateMap,
        },
        systems::Active,
        Fighter,
    },
    input::{Input as FightInput, LEFT},
};

mod battle;
mod fighter;
mod input;
mod util;

const FPS: usize = 60;
const ROLLBACK_DEFAULT: &str = "rollback_default";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStates {
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
        .register_rollback_type::<FrameCount>()
        .with_rollback_schedule(
            Schedule::default()
                .with_stage(
                    ROLLBACK_DEFAULT,
                    SystemStage::parallel()
                        .with_system(
                            buffer_insert_system
                                .run_in_state(GameStates::Fight)
                                .label("InputBuffer"),
                        )
                        .with_system(
                            increment_frame_system
                                .run_in_state(GameStates::Fight)
                                .before("Process")
                                .after("InputBuffer"),
                        )
                        .with_system(
                            process_input_system
                                .run_in_state(GameStates::Fight)
                                .label("Process")
                                .after("InputBuffer"),
                        )
                        .with_system(
                            movement_system
                                .run_in_state(GameStates::Fight)
                                .after("Process"),
                        ),
                )
                .with_stage_after(
                    ROLLBACK_DEFAULT,
                    "Hitbox Stage",
                    SystemStage::parallel()
                        //.with_system(state_system.run_in_state(GameStates::Fight))
                        .with_system(hitbox_component_system.run_in_state(GameStates::Fight)), // .with_system(component_insert_system)
                ),
        )
        .build(&mut app);

    app.add_plugins(DefaultPlugins)
        // .insert_resource(WindowDescriptor {
        //     present_mode: PresentMode::Immediate,
        //     ..default()
        // })
        .add_plugin(JsonAssetPlugin::<SerializedStateVec>::new(&["sl.json"]))
        .add_plugin(JsonAssetPlugin::<FighterData>::new(&["json"]))
        // Game State Systems and Related Plugins
        .add_loopless_state(GameStates::LoadingFight)
        .add_plugin(
            ProgressPlugin::new(GameStates::LoadingFight)
                .continue_to(GameStates::Fight)
                .track_assets(),
        )
        .add_enter_system(GameStates::LoadingFight, load_fighters)
        .add_exit_system(GameStates::LoadingFight, spawn_fighters)
        .add_enter_system(GameStates::Fight, startup.exclusive_system())
        //.add_startup_system(startup.exclusive_system().after("Spawn"))
        //.add_startup_system(spawn_fighters.exclusive_system().label("Spawn"))
        //.add_system(increment_frame_system.run_in_state(GameStates::Fight))
        //.add_system(something_system.run_in_state(GameStates::Fight))
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(EditorPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(sess)
        .insert_resource(SessionType::SyncTestSession)
        .add_plugin(FighterPlugin)
        .register_type::<Movement>()
        .register_type::<InputTransition>()
        // These registers below are purely for inspector
        .register_type::<Player>()
        .register_type::<StateMap>()
        .register_type::<CurrentState>()
        .register_type::<FightState>()
        .register_type::<HitboxData>()
        .register_type::<InputBuffer>()
        //.insert_resource(FrameCount { frame: 0 })
        .insert_resource(Msaa { samples: 4 });

    app.run();
}

fn startup(world: &mut World) {
    // let players = world.get_resource::<PlayerEntities>().unwrap();
    // let player1 = world.entity(players.0).get::<FighterData>().unwrap();

    //let path = format!("../assets/data/fighters/{}/states.sl.json", player1.name.to_lowercase());

    println!("After migrating, does it get here?");

    let deserialized: Vec<SerializedState> =
        serde_json::from_str(include_str!("../assets/data/fighters/tahu/states.sl.json")).unwrap();
    //println!("Deserialized file: {:?}", deserialized);
    // let handle1: Handle<SerializedStateVec>;
    // {
    //     let access = &world.get_resource::<PlayerHandleAccess>().unwrap();
    //     handle1 = access.0.state_list.clone();
    // }
    let mut state_lists = world
        .get_resource_mut::<Assets<SerializedStateVec>>()
        .expect("Can't find this SerializedStateVec");
    //let access = world.remove_resource::<PlayerHandleAccess>().unwrap();
    //let access = world.ge
    //let deserialized: Vec<SerializedState> = state_lists.remove(access.0.state_list).take().unwrap().0;
    //let deserialized = &state_lists.get(&access.0.state_list).unwrap().0;
    // let deserialized = &state_lists.get(&handle1).unwrap().0;

    let player1: Entity;
    let player2: Entity;

    {
        let players = world.get_resource::<PlayerEntities>().unwrap();
        player1 = players.get(1);
        player2 = players.get(2);
    }

    println!("Deserialized file: {:?}", deserialized);

    populate_entities_with_states(world, player1, deserialized);

    /*
    let mut state_map = StateMap::new();

    for mut state in deserialized {
        let name = state
            .debug_name
            .as_ref()
            .unwrap_or(&"State".to_string())
            .clone();
        println!("Name: {}", name);
        let entity = world
            .spawn()
            .insert(Name::new(name))
            .insert_bundle(VisibilityBundle::default())
            //.insert(Player(1))
            //.insert(StateFrame(0))
            .id();

        {
            world.entity_mut(player1).push_children(&[entity]);
        }

        state_map.add_state(state.id, entity);
        let hbox_serialized = state.unsorted_hitboxes.take();
        let mods_serialized = state.modifiers.take();

        let mut state = FightState::from_serialized(state);

        if let Some(hitboxes) = hbox_serialized {
            let mut ordered: HashMap<u16, HashSet<Entity>> = HashMap::new();
            for hitbox in hitboxes {
                let start_frame = hitbox.start_frame;
                let hitbox_entity = world.spawn().insert(hitbox).id();

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

    // let players = world.get_resource::<PlayerEntities>().unwrap();
    //let player1 = world.entity(players.0).get::<FighterData>().unwrap();

    world.entity_mut(player1).insert(state_map);
    */
    // world.get::<CurrentState>(player1).expect("Entity does not have CurrentState component");
    // world.get::<StateMap>(player1).expect("Entity does not have StateMap component");
    // world.get::<Transform>(player1).expect("Entity does not have Transform component");
    // world.get::<FighterData>(player1).expect("Entity does not have FighterData component");
    // world.get::<InputBuffer>(player1).expect("Entity does not have InputBuffer component");
    // world.get::<Fighter>(player1).expect("Entity does not have Fighter component");

    // Get the boxed trait object from the deserialized FightState
    // let modifier= deserialized.get(0).unwrap().modifiers.as_ref().unwrap().get(0).unwrap();
    // let modifier: Box<dyn Reflect> = modifier.clone_value();

    // let type_registry = world.get_resource::<TypeRegistry>().unwrap().clone();
    // let type_registry = type_registry.read();

    // let registration = type_registry.get_with_name(modifier.type_name()).unwrap();
    // let reflect_component = registration.data::<ReflectComponent>().unwrap();

    // let entity = world.spawn().id();
    // reflect_component.add_component(world, entity,&**&modifier);

    // world.entity_mut(entity).insert(Active);
}

fn populate_entities_with_states(
    world: &mut World,
    player: Entity,
    deserialized: Vec<SerializedState>,
) {
    let mut state_map = StateMap::new();

    for mut state in deserialized {
        let name = state
            .debug_name
            .as_ref()
            .unwrap_or(&"State".to_string())
            .clone();
        println!("Name: {}", name);
        let entity = world
            .spawn()
            .insert(Name::new(name))
            .insert_bundle(VisibilityBundle::default())
            //.insert(Player(1))
            //.insert(StateFrame(0))
            .id();

        {
            world.entity_mut(player).push_children(&[entity]);
        }

        state_map.add_state(state.id, entity);
        let hbox_serialized = state.unsorted_hitboxes.take();
        let mods_serialized = state.modifiers.take();

        let mut state = FightState::from_serialized(state);

        if let Some(hitboxes) = hbox_serialized {
            let mut ordered: HashMap<u16, HashSet<Entity>> = HashMap::new();
            for hitbox in hitboxes {
                let start_frame = hitbox.start_frame;
                let hitbox_entity = world.spawn().insert(hitbox).id();

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

    world.entity_mut(player).insert(state_map);
}

// pub fn component_insert_system(
//     query: Query<(Entity, &Player), (With<Active>, With<Movement>)>,
//     frame_count: Res<FrameCount>
// ) {
//     for _ in query.iter() {
//         println!("Second stage: {}", frame_count.frame)
//     }
// }

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}

#[derive(Default, Reflect, Hash, Component)]
#[reflect(Hash)]
pub struct FrameCount {
    pub frame: u32,
}
