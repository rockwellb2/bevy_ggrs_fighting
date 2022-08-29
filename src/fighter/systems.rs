use super::{
    data::{FighterData, Collider, CollisionData, HitEvent},
    state::{
        Active, AdjustFacing, CurrentState, Direction, Facing, HitboxData, HurtboxData,
        InputTransition, Movement, Owner, State, StateFrame, StateMap, HBox, Health,
    },
    Fighter,
};
use bevy::{
    ecs::{reflect::ReflectComponent, entity::EntityMap},
    prelude::{
        Commands, Component, Entity, Query, Res, SpatialBundle, Transform, Visibility, With, ParamSet, Changed, Vec3, EventWriter, EventReader,
    },
    reflect::Reflect,
    utils::{default, HashMap, hashbrown::HashSet}, render::view::visibility, ui::{Style, Val},
};
use bevy_ggrs::Rollback;
use ggrs::InputStatus;
use nalgebra::{Isometry3, Vector3};
use parry3d::{query::intersection_test, shape::Cuboid};

use crate::{
    battle::{PlayerEntities, Lifebar},
    input::{
        ButtonPress, DirectionalInput, Input as FightInput, StateInput, DOWN, DOWN_HELD, LEFT,
        LEFT_HELD, MAP, RAW_DOWN, RAW_LEFT, RAW_LP, RAW_RIGHT, RAW_UP, RIGHT, RIGHT_HELD, UP,
        UP_HELD,
    },
    util::Buffer,
    Player, FPS,
};

pub fn movement_system(
    //state_query: Query<&Parent, (With<Movement>)>,
    //mut fighter_query: Query<(&mut Transform, &FighterData), With<Fighter>>,
    mut fighter_query: Query<
        (
            &mut CurrentState,
            &StateMap,
            &mut Transform,
            &FighterData,
            &InputBuffer,
        ),
        With<Fighter>,
    >,
    state_query: Query<With<Movement>>,
    //inputs: Res<Vec<(FightInput, InputStatus)>>,
    //mut commands: Commands,
    //frame_count: Res<FrameCount>
) {
    //println!("How often does this print?");

    for (mut current, map, mut tf, data, buffer) in fighter_query.iter_mut() {
        //println!("Is this called every frame?");
        let state = map.get(&current.0).expect("State doesn't exist.");
        //let another = state_query.get(*state);

        if let Ok(_) = state_query.get(*state) {
            let input: &Buffer<u32> = &buffer.0;

            if let Some(last) = input.last() {
                let last_unref = *last;
                let unpacked: StateInput = last_unref.into();

                // if unpacked.x == DirectionalInput::None {
                //     current.0 = 0;
                //     return;
                // }

                if *last & LEFT == LEFT || *last & LEFT_HELD == LEFT_HELD {
                    tf.translation.x -= data.walk_speed / FPS as f32;
                } else if *last & RIGHT == RIGHT || *last & RIGHT_HELD == RIGHT_HELD {
                    tf.translation.x += data.walk_speed / FPS as f32;
                }
            }
        }
    }

    // for fighter in state_query.iter() {
    //     if let Ok((mut tf, data)) = fighter_query.get_mut(fighter.0) {
    //         let input: u16 = inputs[0].0.0;
    //         //println!("{:?}", input);

    //         if input & LEFT != 0 {
    //             tf.translation.x -= data.walk_speed / FPS as f32;
    //         }
    //         else if input & RIGHT != 0 {
    //             tf.translation.x += data.walk_speed / FPS as f32;
    //         }
    //     }
    // }

    // for (e, _p) in query.iter() {
    //     println!("First stage: {}", frame_count.frame);
    //     commands.entity(e).insert(Active);

    //     for (mut tf, data) in fighter_query.iter_mut() {
    //         //tf.translation.x -= data.walk_speed;

    //     }
    // }
}

pub fn increment_frame_system(mut query: Query<&mut StateFrame, (With<Player>, With<Fighter>)>) {
    for mut frame in query.iter_mut() {
        //println!("Printing increment frame");
        frame.0 = frame.0.checked_add(1).unwrap_or(1);
    }
}

pub fn process_input_system(
    mut commands: Commands,
    mut query: Query<
        (&mut CurrentState, &StateMap, &InputBuffer, &mut StateFrame),
        (With<Fighter>, With<Player>),
    >,
    state_query: Query<(Option<&InputTransition>, &State)>,

    mut hurtbox_query: Query<(Entity, &HurtboxData, &mut Visibility)>
) {
    for (mut current, map, buffer, mut frame) in query.iter_mut() {
        //println!("Does this print?");

        let state: &Entity = map.get(&current.0).expect("State doesn't exist");

        if let Ok((transitions, s)) = state_query.get(*state) {
            if let Some(transitions) = transitions {
                for (command, to_state) in transitions.0.iter() {
                    if command.compare(&buffer.0) {
                        if let Some(hurtboxes) = &s.hurtboxes {
                            if let Some(set) = hurtboxes.get(&0) {
                                for hurtbox in set {
                                    if let Ok((entity, data, mut visibility)) = hurtbox_query.get_mut(*hurtbox) {
                                        visibility.is_visible = false;
                                        commands.entity(entity).remove::<Active>();
                                    }
                                }
                            }
                        }


                        current.0 = *to_state;
                        frame.0 = 1;
                        return;
                    }
                }
            }

            if let Some(duration) = s.duration {
                if frame.0 > duration {
                    if let Some(hurtboxes) = &s.hurtboxes {
                        if let Some(set) = hurtboxes.get(&0) {
                            for hurtbox in set {
                                if let Ok((entity, data, mut visibility)) = hurtbox_query.get_mut(*hurtbox) {
                                    visibility.is_visible = false;
                                    commands.entity(entity).remove::<Active>();
                                }
                            }
                        }
                    }


                    // TODO: Add component that says which state it should return to
                    current.0 = 0;
                    frame.0 = 1;
                }
            }
        }
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct InputBuffer(pub Buffer<u32>);
//pub struct InputBuffer(pub Buffer<FightInput>);

// #[derive(Default, Component, Reflect)]
// #[reflect(Component)]
// pub struct InputBuffer(pub Vec<FightInput>);

pub fn buffer_insert_system(
    //mut query: Query<&mut InputBuffer, With<Player>>,
    mut query: Query<(&mut InputBuffer, &Player)>,
    inputs: Res<Vec<(FightInput, InputStatus)>>,
) {

    for (mut buffer, player) in query.iter_mut() {
        if player.0 != 1 {
            return
        }
        buffer.0.push(inputs[0].0 .0)
    }
}

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
        With<Fighter>,
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

pub fn adjust_facing_system(
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
    mut hitbox_query: Query<(Entity, &Owner, &mut Active), (With<HitboxData>)>,
    hurtbox_query: Query<(Entity, &Owner), (With<HurtboxData>, With<Active>)>,

    hit_query: Query<(&HitboxData, &Collider, &Transform)>,
    hurt_query: Query<(&HurtboxData, &Collider, &Transform)>,

    mut hit_writer: EventWriter<HitEvent>
) {

    let mut seen_hitboxes: HashMap<Entity, (Isometry3<f32>, Cuboid, HitboxData)> = HashMap::new();
    let mut seen_hurtboxes: HashMap<Entity, (Isometry3<f32>, Cuboid, HurtboxData)> = HashMap::new();
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
                    if hit_data.id >= c.get_attacker_id() {
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

                        println!("Collide!!!!!") 
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
    mut hit_reader: EventReader<HitEvent>,
    mut fighter_query: Query<&mut Health, With<Fighter>>,
    mut hitbox_query: Query<(&mut Active, &HitboxData, &Owner)>
) {
    for hit_event in hit_reader.iter() {
        if let Ok(mut health) = fighter_query.get_mut(hit_event.0.recipient) {
            health.0 = health.0.saturating_sub(hit_event.0.attacker_box.damage);
        }

        for (mut active, _data, owner) in hitbox_query.iter_mut() {
            if owner.0 == hit_event.0.attacker {
                active.0.insert(hit_event.0.recipient);
            }
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