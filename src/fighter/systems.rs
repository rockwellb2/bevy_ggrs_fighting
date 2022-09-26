

use super::{
    data::{FighterData, Collider, CollisionData, HitEvent},
    state::{
        Active, CurrentState, Direction, Facing, HitboxData, HurtboxData, Owner, State, StateFrame, StateMap, HBox, Health, Conditions, InHitstun, Velocity, ProjectileReference, ProjectileData, PlayerAxis,
    },
    Fighter, event::TransitionEvent, modifiers::{Movement, AdjustFacing, CreateObject, Object, Velo, VectorType},
};
use bevy::{
    ecs::{reflect::ReflectComponent, },
    prelude::{
        Commands, Component, Entity, Query, Res, SpatialBundle, Transform, Visibility, With, ParamSet, Changed, Vec3, EventWriter, EventReader, Without, ResMut, Name, Or, Parent, ChangeTrackers,
    },
    reflect::{Reflect, Struct, FromReflect},
    utils::{default, HashMap, hashbrown::HashSet}, ui::{Style, Val}, math::Vec3Swizzles,
};
use bevy_ggrs::{RollbackIdProvider, Rollback};
use ggrs::InputStatus;
use nalgebra::{Isometry3, Vector3};
use parry3d::{query::intersection_test, shape::{Cuboid, Capsule}};


use crate::{
    battle::{PlayerEntities, Lifebar},
    input::{
        Input as FightInput, LEFT,
        LEFT_HELD, RIGHT, RIGHT_HELD,
    },
    util::Buffer,
    Player, FPS,
};

pub fn buffer_insert_system(
    mut query: Query<(&mut InputBuffer, &Player)>,
    inputs: Res<Vec<(FightInput, InputStatus)>>,
) {
    for (mut buffer, player) in query.iter_mut() {
        if player.0 != 1 {
            return
        }
        buffer.0.insert(inputs[0].0 .0)
    }
}



pub fn movement_system(
    query: Query<(&Parent, &Velo)>,
    mut fighter_query: Query<(&StateMap, &CurrentState, &mut Transform, &mut Velocity, &StateFrame, &FighterData, &Facing, &PlayerAxis)>

) {
    for (map, current, mut tf, mut velocity, frame, data, facing, axis) in fighter_query.iter_mut() {
        let s = map.get(&current.0).expect("State doesn't exist");

        if let Ok((_parent, velo)) = query.get(*s) {
            if frame.0 == 1  {
                if let Some(start) = &velo.start_velocity {
                    let mut start: Vec3 = match start {
                        VectorType::Vec(vector) => *vector,
                        VectorType::Variable(var_name) => {
                            let raw = data.field(&var_name).expect("Couldn't get value for field of this name");
                            let variable = f32::from_reflect(raw).expect("Couldn't create f32 from reflected value");

                            Vec3::new(variable, 0., 0.)

                        },
                        VectorType::Warning => panic!(),
                    };

                    start.x *= facing.0.sign();
                    velocity.0 = start;
                }
                
            }
            else {
                if let Some(accel) = &velo.acceleration {
                    let accel: Vec3 = match accel {
                        VectorType::Vec(vector) => *vector,
                        VectorType::Variable(var_name) => {
                            let raw = data.field(&var_name).expect("Couldn't get value for field of this name");
                            let variable = f32::from_reflect(raw).expect("Couldn't create f32 from reflected value");

                            Vec3::new(variable, 0., 0.)

                        },
                        VectorType::Warning => panic!(),
                    };

                    velocity.0 += accel;
                }
            }

            // tf.translation += (velocity.0.x / FPS as f32) * axis.x;
            // tf.translation.y += velocity.0.y / FPS as f32;
            // tf.translation += (velocity.0.z / FPS as f32) * axis.z;

            tf.translation += velocity.0 / FPS as f32;


            tf.translation.y = tf.translation.y.max(0.)


        }
    }
    
}

pub fn increment_frame_system(mut query: Query<&mut StateFrame, Or<(With<Fighter>, With<Active>)>>) {
    for mut frame in query.iter_mut() {
        frame.0 = frame.0.checked_add(1).unwrap_or(1);
    }
}

pub fn hitstun_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CurrentState, &mut StateFrame, &InHitstun), With<Fighter>>
) {
    for (fighter, mut current, mut frame, hitstun) in query.iter_mut() {
        if frame.0 > hitstun.0 {
            frame.0 = 1;
            current.0 = 0;
            commands
                .entity(fighter)
                .remove::<InHitstun>();
        }
    }
}

pub fn process_input_system(
    //mut commands: Commands,
    query: Query<
        (Entity, &CurrentState, &StateMap, &InputBuffer, &StateFrame, &Player, &Facing, &Transform),
        (With<Fighter>, With<Player>),
    >,
    state_query: Query<&State>,
    mut trans_writer: EventWriter<TransitionEvent>
) {
    'fighter: for (fighter, current, map, buffer, frame, player, facing, tf) in query.iter() {


        let state: &Entity = map.get(&current.0).expect("State doesn't exist");

        if let Ok(s) = state_query.get(*state) {
             'transitions: for to_state in state_query.iter_many(&s.transitions) {

                if let Some(all) = &to_state.triggers.0 {
                    let mut meets_conditions = true;
                    'all: for condition in all.iter() {
                        
                        match condition {
                            Conditions::In(n) => {
                                if !n.contains(&current.0) {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            },
                            Conditions::NotIn(n) => {
                                if n == &current.0 {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            },
                            Conditions::Command(command) => {
                                if !command.compare(&buffer.0, facing.0) {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            },
                            Conditions::EndDuration => {
                                if frame.0 <= s.duration.expect("State doesn't have duration") {
                                    meets_conditions = false;
                                    break 'all;
                                }
                            },
                            Conditions::Frame(start_frame, end_frame) => {
                                if let Some(start) = start_frame {
                                    if frame.0 < *start {
                                        meets_conditions = false;
                                        break 'all;
                                    }
                                }

                                if let Some(end) = end_frame {
                                    if frame.0 > *end {
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

                            },
                            // Conditions::OnHit(id, range) => {
                            //     if let Some(id) = id {
                            //         todo!()
                            //     }
                            // },
                        }
                    }

                    if !meets_conditions {
                        continue 'transitions
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
                            },
                            Conditions::NotIn(n) => {
                                if n == &current.0 {
                                    met = false;
                                    break 'conditions;
                                }
                            },
                            Conditions::Command(command) => {
                                if !command.compare(&buffer.0, facing.0) {
                                    met = false;
                                    break 'conditions;
                                }
                            },

                            Conditions::EndDuration => {
                                if frame.0 <= s.duration.expect("State doesn't have duration") {
                                    met = false;
                                    break 'conditions;
                                }
                            },
                            Conditions::Frame(start_frame, end_frame) => {
                                if let Some(start) = start_frame {
                                    if frame.0 < *start {
                                        met = false;
                                        break 'conditions;
                                    }
                                }

                                if let Some(end) = end_frame {
                                    if frame.0 > *end {
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

                            },
                            // Conditions::OnHit(_, _) => todo!(),
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

pub fn transition_system(
    mut commands: Commands,
    mut trans_reader: EventReader<TransitionEvent>,
    mut fighter_query: Query<(Entity, &mut CurrentState, &StateMap, &mut StateFrame, &InputBuffer), With<Fighter>>,
    state_query: Query<&State>,


    mut box_set: ParamSet<(
        Query<&mut Visibility, With<HurtboxData>>,
        Query<(Entity, &Owner, &mut Visibility), (With<Active>, With<HitboxData>)>

    )>
) {
    for event in trans_reader.iter() {
        if let Ok((fighter, mut current, map, mut frame, _buffer)) = fighter_query.get_mut(event.fighter) {
            let entity = map.get(&current.0).expect("State doesn't exist");

            if let Ok(state) = state_query.get(*entity) {
                if let Some(hurt_map) = &state.hurtboxes {
                    if let Some(hurtboxes) = hurt_map.get(&0) {
                        for hurt in hurtboxes {
                            commands.entity(*hurt)
                                .remove::<Active>();
                            
                            if let Ok(mut visibility) = box_set.p0().get_mut(*hurt) {
                                visibility.is_visible = false;
                            }
                        }
                    }
                }

                for (hurt, owner, mut visibility) in box_set.p1().iter_mut() {
                    if owner.0 == fighter {
                        commands.entity(hurt)
                            .remove::<Active>();

                        visibility.is_visible = false;
                    }

                }

                
            }



            println!("Transition {} to {}", current.0, event.to_id);
            
            current.0 = event.to_id;
            frame.0 = 1;
        }
    }

    trans_reader.clear()
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct InputBuffer(pub Buffer);



pub fn hitbox_component_system(
    mut commands: Commands,
    mut fighter_query: Query<
        (
            &CurrentState,
            &StateMap,
            &Transform,
            &StateFrame,
            &InputBuffer,
            &Facing,
        ),
        (With<Fighter>, Without<InHitstun>)
    >,
    state_query: Query<&State>,
    hitbox_query: Query<&HitboxData>,
) {
    for (current, map, tf, frame, _buffer, facing) in fighter_query.iter_mut() {
        let state = map.get(&current.0).expect("State doesn't exist.");

        if let Ok(s) = state_query.get(*state) {
            if let Some(hitboxes) = &s.hitboxes {
                if let Some(set) = hitboxes.get(&frame.0) {
                    for h in set {
                        let hitbox = hitbox_query.get(*h).expect("Hitbox entity does not exist");
                        let mut offset = hitbox.offset;
                        offset.x *= facing.0.sign();

                        let mut transform = tf.clone();
                        transform.translation += offset;

                        commands
                            .entity(*h)
                            .insert(Active(HashSet::new()))
                            .insert_bundle(SpatialBundle {
                                transform,
                                ..default()
                            });
                    }
                }
            }
        }
    }
}

pub fn hitbox_removal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &HitboxData, &Owner, &mut Visibility), With<Active>>,
    fighter_query: Query<&StateFrame, With<Fighter>>,
) {
    for (entity, data, owner, mut visible) in query.iter_mut() {
        let frame = fighter_query.get(owner.0).expect("Owner doesn't exist");

        if frame.0 > data.end_frame {
            visible.is_visible = false;
            commands.entity(entity).remove::<Active>();
        }
    }
}

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
        ),
        With<Fighter>,
    >,
    state_query: Query<&State>,
    hurtbox_query: Query<&HurtboxData>,
) {
    for (current, map, tf, frame, _buffer, facing) in fighter_query.iter_mut() {
        let state = map.get(&current.0).expect("State doesn't exist.");

        if let Ok(s) = state_query.get(*state) {
            if let Some(hurtboxes) = &s.hurtboxes {
                if frame.0 == 1 {
                    if let Some(zero_set) = hurtboxes.get(&0) {
                        for h in zero_set {
                            let hurtbox =
                                hurtbox_query.get(*h).expect("Hurtbox entity does not exist");
                            let mut offset = hurtbox.offset;
                            offset.x *= facing.0.sign();

                            let mut transform = tf.clone();
                            transform.translation += offset;

                            commands
                                .entity(*h)
                                .insert(Active(HashSet::new()))
                                .insert_bundle(SpatialBundle {
                                    transform,
                                    ..default()
                                });
                        }
                    }
                }

                if let Some(set) = hurtboxes.get(&frame.0) {
                    for h in set {
                        let hurtbox = hurtbox_query.get(*h).expect("Hurtbox entity does not exist");
                        let mut offset = hurtbox.offset;
                        offset.x *= facing.0.sign();

                        let mut transform = tf.clone();
                        transform.translation += offset;

                        commands
                            .entity(*h)
                            .insert(Active(HashSet::new()))
                            .insert_bundle(SpatialBundle {
                                transform,
                                ..default()
                            });
                    }
                }
            }
        }
    }
}

pub fn hurtbox_removal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &HurtboxData, &Owner, &mut Visibility), With<Active>>,
    fighter_query: Query<&StateFrame, With<Fighter>>,
) {
    for (entity, data, owner, mut visible) in query.iter_mut() {
        let frame = fighter_query.get(owner.0).expect("Owner doesn't exist");

        if let Some(end_frame) = data.end_frame {
            if frame.0 > end_frame {
                visible.is_visible = false;
                commands.entity(entity).remove::<Active>();
            }
        }
    }
}

// Isn't removed until after stage is over, may be a problem?
pub fn projectile_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Owner, &ProjectileData, &StateFrame, &mut Visibility, &mut Transform, &mut Velocity), With<Active>>,
    mut fighter_query: Query<(&mut ProjectileReference, &Facing)>

) {
    for (projectile, owner, data, frame, mut visibility, mut tf, mut velo) in query.iter_mut() {
        if frame.0 == data.life_frames {
            if let Ok((mut proj_ref, _facing)) = fighter_query.get_mut(owner.0) {
                let ids = proj_ref.projectile_ids.get_mut(&data.name).expect("Projectile is not in ProjectileReference");
                let mut id_iter = ids.iter_mut();

                loop {
                    if let Some((id, in_use)) = id_iter.next() {
                        if projectile == *id {
                            println!("Changing in-use");
                            *in_use = false;
                            break;
                        }
                    }
                    else {
                        panic!();
                    }
                }

                let amount = proj_ref.amount_in_use.get_mut(&data.name).unwrap();
                *amount -= 1;

                visibility.is_visible = false;

                commands.entity(projectile)
                    .remove::<Active>();

            }
        }
        else {
            if let Ok((_, facing)) = fighter_query.get(owner.0) {
                velo.0 += facing.0.sign() * data.acceleration;
                tf.translation += facing.0.sign() * velo.0 / FPS as f32;
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

        if let Ok(_) = state_query.get(*state1) {
            facing1.0 = if tf1.translation.x > tf2.translation.x {
                Direction::Left
            } else {
                Direction::Right
            };
        }

        if let Ok(_) = state_query.get(*state2) {
            facing2.0 = if tf1.translation.x > tf2.translation.x {
                Direction::Right
            } else {
                Direction::Left
            }
        }
    }
}

pub fn object_system(
    mut commands: Commands,

    mut set: ParamSet<(
        Query<(&mut Transform, &mut Visibility, &mut StateFrame)>,
        Query<(Entity, &CurrentState, &StateMap, &StateFrame, &Transform, &mut ProjectileReference, &Facing), With<Fighter>>,
    )>,
    state_query: Query<(&State, &CreateObject)>
) {

    let mut changes: Vec<(Entity, Vec3)> = Vec::new();

    for (_fighter, current, map, frame, tf, mut projectiles, facing) in set.p1().iter_mut() {
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

                        let ids = projectiles.projectile_ids.get_mut(&projectile.name).expect("Projectile name doesn't exist");
                        let mut iter = ids.iter_mut();

                        let id = loop {
                            if let Some((id, in_use)) = iter.next() {
                                if !*in_use {
                                    *in_use = true;
                                    break Some(id);
                                }
                            }
                            else {
                                break None;
                            }
                        };

                        let id = *id.expect("All IDs are in use!");
                        let new_pos = tf.translation + (facing.0.sign() * projectile.start_position);
                        changes.push((id, new_pos));

                        commands.entity(id)
                            .insert(Active(HashSet::new()));
                    }

                },
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

pub fn hbox_position_system<T: HBox>(
    mut set: ParamSet<(
        Query<(&T, &Owner, &mut Transform), With<Active>>, // Hbox Query
        Query<(Entity, &Transform, &Facing), (With<Fighter>, Changed<Transform>)> // Fighter Query
    )>,
) {
    let mut changed: HashMap<Entity, (Vec3, Direction)> = HashMap::new();

    for (player, f_tf, facing) in set.p1().iter() {
        changed.insert(player, (f_tf.translation, facing.0));
    }

    for (hbox, owner, mut h_tf) in set.p0().iter_mut() {
        if let Some((pos, direction)) = changed.get(&owner.0) {
            let mut offset = hbox.get_offset();
            offset.x *= direction.sign();
            h_tf.translation = *pos;
            h_tf.translation += offset;
        }
    }
}

pub fn collision_system(
    mut hitbox_query: Query<(Entity, &Owner, &mut Active), With<HitboxData>>,
    hurtbox_query: Query<(Entity, &Owner), (With<HurtboxData>, With<Active>)>,

    hit_query: Query<(&HitboxData, &Collider, &Transform)>,
    hurt_query: Query<(&HurtboxData, &Collider, &Transform)>,

    mut hit_writer: EventWriter<HitEvent>
) {

    let mut seen_hitboxes: HashMap<Entity, (Isometry3<f32>, Capsule, HitboxData)> = HashMap::new();
    let mut seen_hurtboxes: HashMap<Entity, (Isometry3<f32>, Capsule, HurtboxData)> = HashMap::new();
    // Entities are attacker, recipient
    let mut collisions: HashMap<(Entity, Entity), CollisionData> = HashMap::new();

    for (hitbox, hit_owner, active) in hitbox_query.iter_mut() {
        for (hurtbox, hurt_owner) in hurtbox_query.iter() {
            if active.0.contains(&hurt_owner.0) {
                break;
            }
            if hit_owner != hurt_owner {
                
                let (hit_iso, hit_shape, hit_data) = if seen_hitboxes.contains_key(&hitbox) {
                    seen_hitboxes.get(&hitbox).unwrap().to_owned()
                }
                else {
                    let (data, hit_collider, hit_tf) = hit_query.get(hitbox).unwrap();
                    let hit_vec: Vector3<f32> = hit_tf.translation.into();
                    let iso = Isometry3::from(hit_vec);

                    seen_hitboxes.insert(hitbox, (iso.clone(), hit_collider.shape.clone(), data.clone()));
                    (iso, hit_collider.shape, data.clone())
                };

                let (hurt_iso, hurt_shape, hurt_data) = if seen_hurtboxes.contains_key(&hurtbox) {
                    seen_hurtboxes.get(&hurtbox).unwrap().to_owned()
                }
                else {
                    let (data, hurt_collider, hurt_tf) = hurt_query.get(hurtbox).unwrap();
                    let hurt_vec: Vector3<f32> = hurt_tf.translation.into();
                    let iso = Isometry3::from(hurt_vec);

                    seen_hurtboxes.insert(hurtbox, (iso.clone(), hurt_collider.shape.clone(), data.clone()));
                    (iso, hurt_collider.shape, data.clone())
                };

                if let Some(c) = collisions.get(&(hit_owner.0, hurt_owner.0)) {
                    if hit_data.priority >= c.get_attacker_priority() {
                        break;
                    }
                }


                if let Ok(intersect) = intersection_test(&hit_iso, &hit_shape, &hurt_iso, &hurt_shape) {
                    if intersect { 
                        collisions.insert((hit_owner.0, hurt_owner.0), CollisionData { 
                            attacker_box: hit_data, 
                            attacker: hit_owner.0, 
                            recipient_box: hurt_data, 
                            recipient: hurt_owner.0 
                        });
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
    mut fighter_query: Query<(Entity, &mut Health, &mut StateFrame, &mut CurrentState), With<Fighter>>,
    mut hitbox_query: Query<(&mut Active, &HitboxData, &Owner)>
) {
    for hit_event in hit_reader.iter() {
        if let Ok((fighter, mut health, mut frame, mut current)) = fighter_query.get_mut(hit_event.0.recipient) {
            health.0 = health.0.saturating_sub(hit_event.0.attacker_box.damage);
            commands
                .entity(fighter)
                .insert(InHitstun(hit_event.0.attacker_box.hitstun));

            frame.0 = 1;
            current.0 = 3000;
        }

        for (mut active, _data, owner) in hitbox_query.iter_mut() {
            if owner.0 == hit_event.0.attacker {
                active.0.insert(hit_event.0.recipient);
            }
        }
    }
    
}

pub fn axis_system(
    players: Res<PlayerEntities>,


    mut query: Query<(&Transform, ChangeTrackers<Transform>, &mut PlayerAxis), With<Fighter>>,
) {

    let [(tf1, changed1, mut axis1),
        (tf2, changed2, mut axis2)]
         = query.many_mut(players.as_ref().into());

    if changed1.is_changed() {
        axis2.opponent_pos = tf1.translation;
        match (tf2.translation - tf1.translation).xz().try_normalize() {
            Some(tf) => {
                axis2.x = (tf.x, 0., tf.y).into();
                axis2.z = axis2.x.cross(Vec3::Y);
            },
            None => (),
        }
    }

    if changed2.is_changed() {
        axis1.opponent_pos = tf2.translation;
        match (tf2.translation - tf1.translation).xz().try_normalize() {
            Some(tf) => {
                axis1.x = (tf.x, 0., tf.y).into();
                axis1.z = axis1.x.cross(Vec3::Y);
            },
            None => (),
        }
    }
    
    



}

pub fn ui_lifebar_system(
    mut lifebar_query: Query<(&mut Lifebar, &mut Style, &Player)>,
    fighter_query: Query<&Health, (With<Fighter>, Changed<Health>)>,
    players: Res<PlayerEntities>
) {
    for (mut lifebar, mut style, player) in lifebar_query.iter_mut() {
        if let Ok(health) = fighter_query.get(players.get(player.0)) {
            lifebar.current = health.0;

            let percent = lifebar.health_percent();
            style.size.width = Val::Percent(percent);
        }
    }

}