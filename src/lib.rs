pub mod battle;
pub mod fighter;
pub mod game;
pub mod input;
pub mod util;

use std::{env, net::SocketAddr};

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, ecs::reflect::ReflectComponent, prelude::*,
    reflect::TypeRegistry, utils::HashMap,
};

use bevy_ggrs::{RollbackIdProvider, Rollback};
use parry3d::shape::{Capsule, Cuboid};
use ggrs::Config;

use fighter::state::{State as FightState, CurrentState, ActiveState, PassiveState};
use crate::fighter::state::{SerializedStateVec, SerializedState, StateMap, Owner, HBox};
use crate:: fighter::hit::components::HitboxData;
use crate::fighter::data::Collider;
use crate::battle::{PlayerEntities, PlayerHandleAccess};
use crate::game::RoundState;

pub const FPS: usize = 60;
pub const ROLLBACK_DEFAULT: &str = "rollback_default";


#[derive(Resource)]
pub struct HitboxMap(pub HashMap<u32, (Handle<Mesh>, Collider)>);

#[derive(Resource)]
pub struct GameDebug(pub bool);

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct Player(u8);

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
            let active_or_passive = state.active_type.clone();

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

                    let start_frame = hitbox.get_start_frame();

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

                    let mut e = world.entity_mut(entity);
                    //reflect_component.insert(world, entity, &**&modifier);
                    reflect_component.insert(&mut e, &**&modifier);
                }
            }

            match active_or_passive {
                fighter::state::ActiveOrPassive::Active => {
                    world.entity_mut(entity).insert(ActiveState);
                },
                fighter::state::ActiveOrPassive::Passive => {
                    world.entity_mut(entity).insert(PassiveState);
                },
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
            .insert(CurrentState(*state_map.get(&0).expect("State with ID of 0 doesn't exist")))
            .insert(state_map);
            
    });
}

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


#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = input::Input;
    type State = u8;
    type Address = SocketAddr;
}