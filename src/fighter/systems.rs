#![allow(clippy::type_complexity)]

use super::{
    data::{Collider, CollisionData, FighterData, HitEvent},
    event::TransitionEvent,
    hit::components::{OnHit, AirborneHitstun},
    modifiers::{
        AdjustFacing, CreateObject, InputMet, InputWindowCheck, Object, OnExitSetPos, VectorType,
        Velo, OnExitZeroVelo,
    },
    state::{
        Active, ActiveHitboxes, BoneMap, Conditions, CurrentState, Direction, Exclude, Facing,
        GroundedHitstun, HBox, Health, HurtboxData, Hurtboxes, Owner, PlayerAxis, ProjectileData,
        ProjectileReference, State, StateFrame, StateMap, Velocity, AIR_HITSTUN, GRND_HITSTUN_KB,
    },
    Fighter,
};
use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec3Swizzles,
    prelude::{
        BuildChildren, ChangeTrackers, Changed, Commands, Component, Entity, EulerRot, EventReader,
        EventWriter, GlobalTransform, KeyCode, Name, Or, ParamSet, PbrBundle, Quat, Query, Res,
        ResMut, Transform, Vec3, Visibility, With, Without,
    },
    reflect::{FromReflect, Reflect, Struct},
    ui::{Style, Val},
    utils::{default, hashbrown::HashSet, HashMap},
};
use bevy_ggrs::{PlayerInputs, Rollback, RollbackIdProvider};

use nalgebra::Isometry3;
use parry3d::{
    bounding_volume::{Aabb, BoundingVolume},
    math::Point,
    query::intersection_test,
    shape::Capsule,
};

use bevy::input::Input;

use crate::{
    battle::{HitboxMaterial, Lifebar, MatchCamera, PlayerEntities},
    fighter::hit::components::HitboxData,
    game::{Paused, RoundState},
    util::Buffer,
    GGRSConfig, HitboxMap, Player, FPS,
};

pub fn buffer_insert_system(
    mut query: Query<(&mut InputBuffer, &Player)>,
    inputs: Res<PlayerInputs<GGRSConfig>>,
) {
    for (mut buffer, player) in query.iter_mut() {
        if player.0 != 1 {
            return;
        }
        buffer.0.insert(inputs[0].0 .0)
    }
}

pub fn movement_system(
    query: Query<&Velo>,
    mut fighter_query: Query<(
        &StateMap,
        &CurrentState,
        &mut Transform,
        &mut Velocity,
        &StateFrame,
        &FighterData,
        &Facing,
        &PlayerAxis,
    )>,

    facing_query: Query<&AdjustFacing>
) {
    for (map, current, mut tf, mut velocity, frame, data, facing, axis) in fighter_query.iter_mut()
    {
        let s = map.get(&current.0).expect("State doesn't exist");

        if let Ok(velo) = query.get(*s) {
            if frame.0 == 1 {
                if let Some(start) = &velo.start_velocity {
                    let mut start: Vec3 = match start {
                        VectorType::Vec(vector) => *vector,
                        VectorType::Variable(var_name) => {
                            let raw = data
                                .field(var_name)
                                .expect("Couldn't get value for field of this name");
                            let variable = f32::from_reflect(raw)
                                .expect("Couldn't create f32 from reflected value");

                            Vec3::new(variable, 0., 0.)
                        }
                        VectorType::Warning => panic!(),
                    };

                    start.x *= facing.0.sign();
                    velocity.0 = start;
                }
            } else {
                if let Some(accel) = &velo.acceleration {
                    let mut accel: Vec3 = match accel {
                        VectorType::Vec(vector) => *vector,
                        VectorType::Variable(var_name) => {
                            let raw = data
                                .field(var_name)
                                .expect("Couldn't get value for field of this name");
                            let variable = f32::from_reflect(raw)
                                .expect("Couldn't create f32 from reflected value");

                            Vec3::new(variable, 0., 0.)
                        }
                        VectorType::Warning => panic!(),
                    };
                    accel.x *= facing.0.sign();
                    velocity.0 += accel;
                }
            }

            // tf.translation += (velocity.0.x / FPS as f32) * axis.x;
            // tf.translation.y += velocity.0.y / FPS as f32;
            // tf.translation += (velocity.0.z / FPS as f32) * axis.z;

            // //tf.translation += velocity.0 / FPS as f32;

            // tf.translation.y = tf.translation.y.max(0.);

            // tf.look_at(axis.opponent_pos, Vec3::Y);
            // //tf.rotate_axis(Vec3::Y, FRAC_PI_2);
        }

        tf.translation += (velocity.0.x / FPS as f32) * axis.x;
        tf.translation.y += velocity.0.y / FPS as f32;
        tf.translation += (velocity.0.z / FPS as f32) * axis.z;

        //tf.translation += velocity.0 / FPS as f32;

        tf.translation.y = tf.translation.y.max(0.);
        
        if facing_query.get(*s).is_ok() {
            let mut opp_pos = axis.opponent_pos;
            opp_pos.y = tf.translation.y;
            tf.look_at(opp_pos, Vec3::Y);
        }
        //tf.rotate_axis(Vec3::Y, FRAC_PI_2);
    }
}

pub fn increment_frame_system(
    mut query: Query<&mut StateFrame, Or<(With<Fighter>, With<Active>)>>,
) {
    for mut frame in query.iter_mut() {
        frame.0 = frame.0.checked_add(1).unwrap_or(1);
    }
}

pub fn hitstun_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CurrentState, &mut StateFrame, &mut Velocity, Option<&GroundedHitstun>, Option<&AirborneHitstun>, &mut Transform), With<Fighter>>,
) {
    for (fighter, mut current, mut frame, mut velo, hitstun, airborne, mut tf ) in query.iter_mut() {
        if let Some(hitstun) = hitstun {
            if frame.0 > hitstun.0 {
                frame.0 = 1;
                current.0 = 0;
                velo.0 = Vec3::ZERO;
                commands.entity(fighter).remove::<GroundedHitstun>();
            }
        }

        if airborne.is_some() {
            if tf.translation.y <= 0. && frame.0 != 1 {
                frame.0 = 1;
                tf.translation.y = 0.;
                current.0 = 0;
                velo.0 = Vec3::ZERO;
                commands.entity(fighter).remove::<AirborneHitstun>();
            }
        }
        
    }

    
}

#[allow(clippy::type_complexity)]
pub fn process_input_system(
    //mut commands: Commands,
    query: Query<
        (
            Entity,
            &CurrentState,
            &StateMap,
            &InputBuffer,
            &StateFrame,
            &Player,
            &Facing,
            &Transform,
        ),
        (With<Fighter>, With<Player>),
    >,
    state_query: Query<(Entity, &State)>,
    mut trans_writer: EventWriter<TransitionEvent>,

    input_met_mod_query: Query<&InputMet>,
) {
    'fighter: for (fighter, current, map, buffer, frame, player, facing, tf) in query.iter() {
        let state: &Entity = map.get(&current.0).expect("State doesn't exist");

        // if current.0 == 250 {
        //     let q: StateInput = buffer.0.get(0).unwrap().into();
        //     println!("250 Input: {:?}", q);
        // }

        if let Ok((_, s)) = state_query.get(*state) {
            'transitions: for (to_state_entity, to_state) in state_query.iter_many(&s.transitions) {
                if let Some(all) = &to_state.triggers.0 {
                    let mut meets_conditions = true;
                    'all: for condition in all.iter() {
                        match condition {
                            Conditions::In(n) => {
                                if !n.contains(&current.0) {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            }
                            Conditions::NotIn(n) => {
                                if n == &current.0 {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            }
                            Conditions::Command(command) => {
                                if !command.compare(&buffer.0, facing.0) {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            }
                            Conditions::EndDuration => {
                                if frame.0 <= s.duration.expect("State doesn't have duration") {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            }
                            Conditions::Frame(window) => {
                                if let Ok(start) = window.try_get_start_frame() {
                                    if frame.0 < start {
                                        meets_conditions = false;
                                        break 'all;
                                    }
                                }

                                if let Ok(end) = window.try_get_end_frame() {
                                    if frame.0 > end {
                                        meets_conditions = false;
                                        break 'all;
                                    }
                                }
                            }
                            Conditions::ReachGround => {
                                if tf.translation.y > 0. {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            }
                            Conditions::InputWindowCon(enact_frame) => {
                                let met_mod = input_met_mod_query
                                    .get(*state)
                                    .expect("State entity doesn't have InputMet component");

                                if !met_mod.0 || *enact_frame != frame.0 {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            }
                            // Conditions::OnHit(id, range) => {
                            //     if let Some(id) = id {
                            //         todo!()
                            //     }
                            // },
                        }
                    }

                    if !meets_conditions {
                        continue 'transitions;
                    }
                }

                let mut others = true;

                'set: for con_set in to_state.triggers.1.iter() {
                    let mut met = true;
                    others = false;
                    'conditions: for conditions in con_set.iter() {
                        match conditions {
                            Conditions::In(n) => {
                                if !n.contains(&current.0) {
                                    met = false;
                                    break 'conditions;
                                }
                            }
                            Conditions::NotIn(n) => {
                                if n == &current.0 {
                                    met = false;
                                    break 'conditions;
                                }
                            }
                            Conditions::Command(command) => {
                                if !command.compare(&buffer.0, facing.0) {
                                    met = false;
                                    break 'conditions;
                                }
                            }

                            Conditions::EndDuration => {
                                if frame.0 <= s.duration.expect("State doesn't have duration") {
                                    met = false;
                                    break 'conditions;
                                }
                            }
                            Conditions::Frame(window) => {
                                if let Ok(start) = window.try_get_start_frame() {
                                    if frame.0 < start {
                                        met = false;
                                        break 'conditions;
                                    }
                                }

                                if let Ok(end) = window.try_get_end_frame() {
                                    if frame.0 > end {
                                        met = false;
                                        break 'conditions;
                                    }
                                }
                            }
                            Conditions::ReachGround => {
                                if tf.translation.y > 0. {
                                    met = false;
                                    break 'conditions;
                                }
                            }
                            Conditions::InputWindowCon(enact_frame) => {
                                let met_mod = input_met_mod_query
                                    .get(*state)
                                    .expect("State entity doesn't have InputMet component");

                                if !met_mod.0 || *enact_frame != frame.0 {
                                    met = false;
                                    break 'conditions;
                                }
                            } // Conditions::OnHit(_, _) => todo!(),
                        }
                    }
                    if met {
                        trans_writer.send(TransitionEvent::new(fighter, to_state.id));
                        break 'transitions;
                    }
                }

                if others {
                    trans_writer.send(TransitionEvent::new(fighter, to_state.id));
                    break 'transitions;
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn transition_system(
    mut commands: Commands,
    mut trans_reader: EventReader<TransitionEvent>,
    mut fighter_query: Query<
        (
            Entity,
            &mut CurrentState,
            &StateMap,
            &mut StateFrame,
            &InputBuffer,
            &BoneMap,
            &mut Velocity
        ),
        With<Fighter>,
    >,
    state_query: Query<&State>,
    set_pos_query: Query<&OnExitSetPos>,
    zero_velo_query: Query<&OnExitZeroVelo>,
    mut input_met_query: Query<&mut InputMet>,

    mut transform_set: ParamSet<(Query<&GlobalTransform>, Query<&mut Transform>)>,
) {
    for event in trans_reader.iter() {
        if let Ok((fighter, mut current, map, mut frame, _buffer, bone_map, mut velo)) =
            fighter_query.get_mut(event.fighter)
        {
            println!("Transition {} to {}", current.0, event.to_id);

            let state = map.get(&current.0).expect("State doesn't exist");

            // OnExitSetPos transition
            if let Ok(set_pos) = set_pos_query.get(*state) {
                let bone = bone_map.0.get(&set_pos.bone).expect("Bone doesn't exist");
                let query_global = transform_set.p0();
                let global = query_global
                    .get(*bone)
                    .expect("Bone doesn't have global transform");
                let pos = global.translation();

                let mut query_trans = transform_set.p1();
                let mut tf = query_trans
                    .get_mut(fighter)
                    .expect("Fighter doesn't have transform component");

                tf.translation.x = pos.x;
                tf.translation.z = pos.z;
            }

            // OnExitZeroVelo 
            if zero_velo_query.get(*state).is_ok() {
                velo.0 = Vec3::ZERO;
            }

            // InputMet reset
            if let Ok(mut met) = input_met_query.get_mut(*state) {
                met.0 = false;
            }

            current.0 = event.to_id;
            frame.0 = 1;
        }
    }

    trans_reader.clear()
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct InputBuffer(pub Buffer);

#[allow(clippy::type_complexity)]
pub fn hitbox_component_system(
    mut commands: Commands,
    mut fighter_query: Query<
        (
            Entity,
            &CurrentState,
            &StateMap,
            &Transform,
            &StateFrame,
            &InputBuffer,
            &Facing,
            &PlayerAxis,
            &mut ActiveHitboxes,
        ),
        (With<Fighter>, Without<GroundedHitstun>),
    >,
    state_query: Query<&State>,
    hitbox_query: Query<&HitboxData>,

    mut rip: ResMut<RollbackIdProvider>,
    hitbox_material: Res<HitboxMaterial>,
    hitbox_resource: Res<HitboxMap>,
) {
    for (entity, current, map, tf, frame, _buffer, facing, axis, mut active_hits) in
        fighter_query.iter_mut()
    {
        let state = map.get(&current.0).expect("State doesn't exist.");

        if let Ok(s) = state_query.get(*state) {
            if let Some(hitboxes) = &s.hitboxes {
                if let Some(set) = hitboxes.get(&frame.0) {
                    for hitbox in set {
                        let hit_collider = hitbox_resource
                            .0
                            .get(&hitbox.global_id.expect("GlobalID doesn't exist"))
                            .expect("No Hitbox found for given GlobalID");

                        let h = commands
                            .entity(hitbox.bone_entity.expect("Bone entity doesn't exist"))
                            .add_children(|parent| {
                                parent
                                    .spawn(PbrBundle {
                                        transform: Transform {
                                            translation: hitbox.offset,
                                            rotation: Quat::from_euler(
                                                EulerRot::default(),
                                                0.,
                                                hitbox.rotation.0,
                                                hitbox.rotation.1,
                                            ),
                                            ..default()
                                        },
                                        mesh: hit_collider.0.clone(),
                                        material: hitbox_material.0.clone(),
                                        ..default()
                                    })
                                    .insert(hitbox.clone())
                                    .insert(Rollback::new(rip.next_id()))
                                    .insert(hit_collider.1.clone())
                                    .insert(Owner(entity))
                                    .insert(Exclude(HashSet::new()))
                                    .id()
                            });

                        active_hits.0.push(h);
                    }
                }
            }
        }
    }
}

pub fn hitbox_removal_system(
    mut commands: Commands,
    query: Query<(Entity, &HitboxData, &Owner)>,
    fighter_query: Query<&StateFrame, With<Fighter>>,
) {
    for (entity, data, owner) in query.iter() {
        let frame = fighter_query.get(owner.0).expect("Owner doesn't exist");

        if frame.0 > data.get_end_frame() {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn hurtbox_component_system(
    mut commands: Commands,
    mut fighter_query: Query<
        (
            &CurrentState,
            &StateMap,
            &Transform,
            &StateFrame,
            &InputBuffer,
            &Facing,
            &PlayerAxis,
        ),
        With<Fighter>,
    >,
    state_query: Query<&State>,
    hurtbox_query: Query<&HurtboxData>,
) {
    // for (current, map, tf, frame, _buffer, facing, axis) in fighter_query.iter_mut() {
    //     let state = map.get(&current.0).expect("State doesn't exist.");

    //     if let Ok(s) = state_query.get(*state) {
    //         if let Some(hurtboxes) = &s.hurtboxes {
    //             if frame.0 == 1 {
    //                 if let Some(zero_set) = hurtboxes.get(&0) {
    //                     for h in zero_set {
    //                         let hurtbox =
    //                             hurtbox_query.get(*h).expect("Hurtbox entity does not exist");
    //                         let offset = hurtbox.offset;

    //                         let mut transform = Transform::from_translation(tf.translation);
    //                         transform.rotate_x(hurtbox.rotation.0);
    //                         transform.rotate_z(hurtbox.rotation.1);
    //                         transform.rotate(tf.rotation);
    //                         transform.translation.y = 0.;
    //                         transform.translation += offset.x * axis.x;
    //                         transform.translation += offset.z * axis.z;
    //                         transform.translation.y += offset.y;

    //                         commands
    //                             .entity(*h)
    //                             .insert(Active(HashSet::new()))
    //                             .insert_bundle(SpatialBundle {
    //                                 transform,
    //                                 ..default()
    //                             });
    //                     }
    //                 }
    //             }
}

pub fn hurtbox_removal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &HurtboxData, &Owner, &mut Visibility), With<Active>>,
    fighter_query: Query<&StateFrame, With<Fighter>>,
) {
    // for (entity, data, owner, mut visible) in query.iter_mut() {
    //     let frame = fighter_query.get(owner.0).expect("Owner doesn't exist");

    //     if let Some(end_frame) = data.end_frame {
    //         if frame.0 > end_frame {
    //             visible.is_visible = false;
    //             commands.entity(entity).remove::<Active>();
    //         }
    //     }
    // }
}

// Isn't removed until after stage is over, may be a problem?
#[allow(clippy::type_complexity)]
pub fn projectile_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Owner,
            &ProjectileData,
            &StateFrame,
            &mut Visibility,
            &mut Transform,
            &mut Velocity,
            &PlayerAxis,
        ),
        With<Active>,
    >,
    mut fighter_query: Query<(&mut ProjectileReference, &Facing)>,
) {
    for (projectile, owner, data, frame, mut visibility, mut tf, mut velo, axis) in query.iter_mut()
    {
        if frame.0 == data.life_frames {
            if let Ok((mut proj_ref, _facing)) = fighter_query.get_mut(owner.0) {
                let ids = proj_ref
                    .projectile_ids
                    .get_mut(&data.name)
                    .expect("Projectile is not in ProjectileReference");
                let mut id_iter = ids.iter_mut();

                loop {
                    if let Some((id, in_use)) = id_iter.next() {
                        if projectile == *id {
                            //println!("Changing in-use");
                            *in_use = false;
                            break;
                        }
                    } else {
                        panic!();
                    }
                }

                let amount = proj_ref.amount_in_use.get_mut(&data.name).unwrap();
                *amount -= 1;

                visibility.is_visible = false;

                commands.entity(projectile).remove::<Active>();
            }
        } else {
            if let Ok((_, facing)) = fighter_query.get(owner.0) {
                velo.0 += facing.0.sign() * data.acceleration;
                //tf.translation += facing.0.sign() * velo.0 / FPS as f32;

                tf.translation += (velo.0.x / FPS as f32) * axis.x;
                tf.translation += (velo.0.z / FPS as f32) * axis.z;
                tf.translation.y += velo.0.y / FPS as f32;
            }
        }
    }
}

pub fn adjust_facing_system(
    mut commands: Commands,

    players: Res<PlayerEntities>,
    mut fighter_query: Query<(&CurrentState, &StateMap, &Transform, &mut Facing), With<Fighter>>,
    state_query: Query<With<AdjustFacing>, With<State>>,
) {
    let player1 = players.get(1);
    let player2 = players.get(2);

    if let Ok([(current1, map1, tf1, mut facing1), (current2, map2, tf2, mut facing2)]) =
        fighter_query.get_many_mut([player1, player2])
    {
        let state1 = map1.get(&current1.0).unwrap();
        let state2 = map2.get(&current2.0).unwrap();

        if state_query.get(*state1).is_ok() {
            facing1.0 = if tf1.translation.x > tf2.translation.x {
                Direction::Left
            } else {
                Direction::Right
            };
        }

        if state_query.get(*state2).is_ok() {
            facing2.0 = if tf1.translation.x > tf2.translation.x {
                Direction::Right
            } else {
                Direction::Left
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn object_system(
    mut commands: Commands,

    mut set: ParamSet<(
        Query<(&mut Transform, &mut Visibility, &mut StateFrame)>,
        Query<
            (
                Entity,
                &CurrentState,
                &StateMap,
                &StateFrame,
                &Transform,
                &mut ProjectileReference,
                &Facing,
                &PlayerAxis,
            ),
            With<Fighter>,
        >,
    )>,
    state_query: Query<(&State, &CreateObject)>,
) {
    let mut changes: Vec<(Entity, Vec3)> = Vec::new();

    for (_fighter, current, map, frame, tf, mut projectiles, facing, axis) in set.p1().iter_mut() {
        let s = map.get(&current.0).expect("State does not exist");

        if let Ok((_state, create_object)) = state_query.get(*s) {
            match &create_object.0 {
                Object::Projectile(projectile) => {
                    if projectile.spawn_frame == frame.0 {
                        let amount = projectiles.amount_in_use.get_mut(&projectile.name).unwrap();

                        if *amount >= projectile.max {
                            continue;
                        }

                        *amount += 1;

                        let ids = projectiles
                            .projectile_ids
                            .get_mut(&projectile.name)
                            .expect("Projectile name doesn't exist");
                        let mut iter = ids.iter_mut();

                        let id = loop {
                            if let Some((id, in_use)) = iter.next() {
                                if !*in_use {
                                    *in_use = true;
                                    break Some(id);
                                }
                            } else {
                                break None;
                            }
                        };

                        let id = *id.expect("All IDs are in use!");

                        let mut new_pos = tf.translation;
                        new_pos += projectile.start_position.x * axis.x;
                        new_pos.y = projectile.start_position.y;
                        new_pos += projectile.start_position.z * axis.z;



                        changes.push((id, new_pos));

                        commands
                            .entity(id)
                            .insert(axis.clone())
                            .insert(Active(HashSet::new()));
                    }
                }
                Object::None => panic!(),
            }
        }
    }

    changes.into_iter().for_each(|(entity, pos)| {
        if let Ok((mut tf, mut visibility, mut frame)) = set.p0().get_mut(entity) {
            tf.translation = pos;
            visibility.is_visible = true;
            frame.0 = 1;
        }
    });
}

#[allow(clippy::type_complexity)]
pub fn hbox_position_system<T: HBox>(
    mut set: ParamSet<(
        Query<(&T, &Owner, &mut Transform), With<Active>>, // Hbox Query
        Query<(Entity, &Transform, &Facing, &PlayerAxis), (With<Fighter>, Changed<Transform>)>, // Fighter Query
    )>,
) {
    let mut changed: HashMap<Entity, (Vec3, Direction, PlayerAxis)> = HashMap::new();

    for (player, f_tf, facing, axis) in set.p1().iter() {
        changed.insert(player, (f_tf.translation, facing.0, axis.clone()));
    }

    for (hbox, owner, mut h_tf) in set.p0().iter_mut() {
        if let Some((pos, direction, axis)) = changed.get(&owner.0) {
            let mut offset = hbox.get_offset();
            //offset.x *= direction.sign();
            h_tf.translation = *pos;

            h_tf.translation += axis.x * offset.x;
            h_tf.translation += axis.z * offset.z;
            h_tf.translation.y += offset.y;

            //h_tf.translation += offset;
        }
    }
}

pub fn collision_system(
    hitbox_query: Query<(Entity, &Owner, &Exclude), With<HitboxData>>,
    hurtbox_query: Query<(Entity, &Owner, &Name), With<HurtboxData>>,

    hit_query: Query<(Entity, &HitboxData, &Collider, &GlobalTransform)>,
    hurt_query: Query<(Entity, &HurtboxData, &Collider, &GlobalTransform)>,

    fighter_query: Query<&Hurtboxes, With<Fighter>>,

    mut hit_writer: EventWriter<HitEvent>,

    players: Res<PlayerEntities>,
) {
    // let mut seen_hitboxes: HashMap<Entity, (Isometry3<f32>, Capsule, HitboxData)> = HashMap::new();
    // let mut seen_hurtboxes: HashMap<Entity, (Isometry3<f32>, Capsule, HurtboxData)> = HashMap::new();
    // Entities are attacker, recipient
    let mut collisions: HashMap<(Entity, Entity), CollisionData> = HashMap::new();

    let mut hitboxes_1: Vec<Entity> = Vec::new();
    let mut hitboxes_2: Vec<Entity> = Vec::new();

    for (hit_entity, owner, exclude) in hitbox_query.iter() {
        if owner.0 == players.get(1) && !exclude.0.contains(&players.get(2)) {
            hitboxes_1.push(hit_entity);
        } else if owner.0 == players.get(2) && !exclude.0.contains(&players.get(1)) {
            hitboxes_2.push(hit_entity);
        }
    }

    if let Ok([hurtboxes1, hurtboxes2]) = fighter_query.get_many(players.as_ref().into()) {
        if !hitboxes_1.is_empty() {
            let mut hurt_grouping: Vec<(Isometry3<f32>, Capsule, HurtboxData)> = Vec::new();
            let mut hurt_points: Vec<Point<f32>> = Vec::new();

            for (hurt_ent, hurt_data, hurt_shape, hurt_tf) in
                hurt_query.iter_many(hurtboxes2.list())
            {
                let transform: Transform = hurt_tf.compute_transform();
                let iso: Isometry3<f32> = (transform.translation, transform.rotation).into();
                let capsule: Capsule = hurt_shape.into();
                let aabb = Capsule::aabb(&capsule, &iso);
                hurt_points.push(aabb.mins);
                hurt_points.push(aabb.maxs);
                hurt_grouping.push((iso, capsule, hurt_data.clone()));
            }

            let comp_aabb = Aabb::from_points(&hurt_points);

            'hitbox_loop: for (hit_ent, hitbox1, collider1, tf1) in hit_query.iter_many(&hitboxes_1)
            {
                let capsule: Capsule = collider1.into();
                let transform: Transform = tf1.compute_transform();
                let iso: Isometry3<f32> = (transform.translation, transform.rotation).into();

                let aabb = capsule.aabb(&iso);

                if comp_aabb.intersects(&aabb) {
                    for (hurt_iso, hurt_capsule, hurt_data) in &hurt_grouping {
                        if let Ok(intersect) =
                            intersection_test(&iso, &capsule, hurt_iso, hurt_capsule)
                        {
                            if intersect {
                                collisions.insert(
                                    (players.get(1), players.get(2)),
                                    CollisionData {
                                        attacker_box: hitbox1.clone(),
                                        attacker: players.get(1),
                                        recipient_box: hurt_data.clone(),
                                        recipient: players.get(2),
                                    },
                                );
                                continue 'hitbox_loop;
                            }
                        }
                    }
                }
            }
        }
    }

    for (_, collision) in collisions {
        hit_writer.send(HitEvent(collision));
    }
}

pub fn hit_event_system(
    mut commands: Commands,
    mut hit_reader: EventReader<HitEvent>,
    mut fighter_query: Query<
        (
            Entity,
            &mut Health,
            &mut StateFrame,
            &mut CurrentState,
            &mut Velocity,
            &Facing,
        ),
        With<Fighter>,
    >,
    mut hitbox_query: Query<(&mut Exclude, &HitboxData, &Owner)>,
) {
    for hit_event in hit_reader.iter() {
        if let Ok((fighter, mut health, mut frame, mut current, mut velo, facing)) =
            fighter_query.get_mut(hit_event.0.recipient)
        {
            health.0 = health.0.saturating_sub(hit_event.0.attacker_box.damage);

            match hit_event.0.attacker_box.on_hit {
                OnHit::Launch(kb) => {
                    commands
                        .entity(fighter)
                        .insert(AirborneHitstun);
                
                    
                    frame.0 = 1;
                    current.0 = AIR_HITSTUN;

                    let mut knockback = kb;
                    knockback.x *= facing.0.sign();
                    velo.0 = knockback;

                    assert!(knockback.y > 0.);

                },
                OnHit::Grounded { kb, hitstun } => {
                    commands
                        .entity(fighter)
                        .insert(GroundedHitstun(hitstun));

                    frame.0 = 1;
                    current.0 = GRND_HITSTUN_KB;

                    let mut knockback = kb;
                    knockback.x *= facing.0.sign();
                    velo.0 = knockback;
                },
                OnHit::Stun(stun) => {

                }
            }

            // let mut knockback = hit_event.0.attacker_box.knockback;
            // knockback.x *= facing.0.sign();

            // velo.0 = knockback;
        }

        for (mut exclude, _data, owner) in hitbox_query.iter_mut() {
            if owner.0 == hit_event.0.attacker {
                exclude.0.insert(hit_event.0.recipient);
            }
        }
    }
}

pub fn axis_system(
    players: Res<PlayerEntities>,
    mut query: Query<(&Transform, ChangeTrackers<Transform>, &mut PlayerAxis), With<Fighter>>,
) {
    let [(tf1, changed1, mut axis1), (tf2, changed2, mut axis2)] =
        query.many_mut(players.as_ref().into());

    if changed1.is_changed() || changed2.is_changed() {
        axis2.opponent_pos = tf1.translation;
        match (tf1.translation - tf2.translation).xz().try_normalize() {
            Some(tf) => {
                axis2.x = (tf.x, 0., tf.y).into();
                axis2.z = axis2.x.cross(Vec3::Y);
            }
            None => (),
        }

        axis1.opponent_pos = tf2.translation;
        match (tf2.translation - tf1.translation).xz().try_normalize() {
            Some(tf) => {
                axis1.x = (tf.x, 0., tf.y).into();
                axis1.z = axis1.x.cross(Vec3::Y);
            }
            None => (),
        }
    }
}

pub fn ui_lifebar_system(
    mut lifebar_query: Query<(&mut Lifebar, &mut Style, &Player)>,
    fighter_query: Query<&Health, (With<Fighter>, Changed<Health>)>,
    players: Res<PlayerEntities>,
) {
    for (mut lifebar, mut style, player) in lifebar_query.iter_mut() {
        if let Ok(health) = fighter_query.get(players.get(player.0)) {
            lifebar.current = health.0;

            let percent = lifebar.health_percent();
            style.size.width = Val::Percent(percent);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn camera_system(
    mut set: ParamSet<(
        Query<&mut Transform, With<MatchCamera>>,
        Query<(&Transform, ChangeTrackers<Transform>)>,
    )>,

    players: Res<PlayerEntities>,
) {
    let player_query = set.p1();
    let [(tf1, change1), (tf2, change2)] = player_query.many(players.as_ref().into());

    if change1.is_changed() || change2.is_changed() {
        let mut mid = tf1.translation.lerp(tf2.translation, 0.5);
        let direction = (tf1.translation - tf2.translation).xz();
        let direction: Vec3 = (direction.x, 0., direction.y).into();
        let perp = direction.cross(Vec3::Y);

        // let tf1 = tf1.translation.clone();
        // let tf2 = tf2.translation.clone();

        if let Ok(mut cam_tf) = set.p0().get_single_mut() {
            cam_tf.translation = mid + perp * -2.;
            mid.y = 1.2;
            cam_tf.translation.y = 1.2;
            cam_tf.look_at(mid, Vec3::Y);
        }
    }
}

pub fn pause_system(
    input: Res<Input<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut state: ResMut<RoundState>,
    mut frame_query: Query<&mut StateFrame>,
) {
    if input.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }

    if paused.0 {
        if input.just_pressed(KeyCode::Right) {
            *state = RoundState::AdvanceFrame;
            for mut frame in frame_query.iter_mut() {
                frame.0 += 1;
            }
        } else {
            *state = RoundState::Paused;
        }
    } else {
        *state = RoundState::Round;
    }
}

pub fn last_debug_system(paused: Res<Paused>, mut state: ResMut<RoundState>) {
    if paused.0 {
        *state = RoundState::Paused;
    }
}

pub fn modifier_input_check(
    mut query: Query<(&Owner, &State, &mut InputMet, &InputWindowCheck)>,
    fighter_query: Query<(&InputBuffer, &Facing, &StateFrame, &CurrentState, &StateMap)>,
) {
    // TODO: Check and Met need to be on separate entities
    for (buffer, facing, frame, current, map) in fighter_query.iter() {
        let s: &Entity = map.get(&current.0).expect("State doesn't exist");

        if let Ok((_, state, mut met, check)) = query.get_mut(*s) {
            if !met.0
                && frame.0 >= check.window.get_start_frame()
                && frame.0 <= check.window.get_end_frame()
                && check.command_input.compare(&buffer.0, facing.0)
            {
                met.0 = true;
            }
        }
    }

    // for (owner, state, mut met, check) in query.iter_mut() {
    //     let (buffer, facing, frame, _, _) = fighter_query.get(owner.get()).expect("Fighter doesn't have InputBuffer or Facing component");

    //     if !met.0 && frame.0 >= check.window_start && frame.0 <= check.window_end && check.command_input.compare(&buffer.0, facing.0)
    //     {
    //         println!("Met at frame {}", frame.0);

    //         met.0 = true;

    //         for (index, i) in buffer.0.iter().enumerate() {
    //             let i: StateInput = i.into();
    //             println!("{}: {:?}", index, i)
    //         }

    //     }
    // }
}
