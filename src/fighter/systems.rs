



use super::{
    data::FighterData,
    state::{CurrentState, Movement, StateFrame, StateMap, State, InputTransition, HitboxData},
    Fighter,
};
use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Component, Entity, Query, Res, Transform, With, Commands, SpatialBundle},
    reflect::{Reflect},
    utils::default
};
use bevy_ggrs::Rollback;
use ggrs::InputStatus;

use crate::{
    input::{Input as FightInput, LEFT, RIGHT}, Player, FPS, util::Buffer,
};

// Component to attach to an entity when its state should actively query systems
#[derive(Component, Default, Reflect)]
pub struct Active;

pub fn movement_system(
    //state_query: Query<&Parent, (With<Movement>)>,
    //mut fighter_query: Query<(&mut Transform, &FighterData), With<Fighter>>,
    mut fighter_query: Query<
        (
            &CurrentState,
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


    for (current, map, mut tf, data, buffer) in fighter_query.iter_mut() {
        //println!("Is this called every frame?");
        let state = map.get(&current.0).expect("State doesn't exist.");
        //let another = state_query.get(*state);

        if let Ok(_) = state_query.get(*state) {
            // //let input: u16 = inputs[0].0 .0;

            //println!("Migration movement");

            // if input & LEFT != 0 {
            //     tf.translation.x -= data.walk_speed / FPS as f32;
            // } else if input & RIGHT != 0 {
            //     tf.translation.x += data.walk_speed / FPS as f32;
            // }
            //println!("Printing movement");

            let input: &Buffer<FightInput> = &buffer.0;

            if let Some(last) = input.last() {
                if last.0 & LEFT != 0 {
                    tf.translation.x -= data.walk_speed / FPS as f32;
                } else if last.0 & RIGHT != 0 {
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

pub fn increment_frame_system(mut query: Query<&mut StateFrame, With<Player>>) {
    for mut frame in query.iter_mut() {
        //println!("Printing increment frame");
        frame.0 = frame.0.checked_add(1).unwrap_or(1);
    }
}

pub fn process_input_system(
    mut query: Query<(&mut CurrentState, &StateMap, &InputBuffer, &mut StateFrame), (With<Fighter>, With<Player>)>,
    state_query: Query<(Option<&InputTransition>, &State)>
) {
    for (mut current, map, buffer, mut frame) in query.iter_mut() {
        //println!("Does this print?");
        
        let state: &Entity = map.get(&current.0).expect("State doesn't exist");

        if let Ok((transitions, s)) = state_query.get(*state) {
            //println!("Always prints after the above");
            if let Some(transitions) = transitions {
                for (command, u16) in transitions.0.iter() {
                    if command.compare(&buffer.0) {
                        current.0 = *u16;
                        frame.0 = 1;
                        return;
                    }
                }
            }

            if let Some(duration) = s.duration {
                if frame.0 > duration {
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
pub struct InputBuffer(pub Buffer<FightInput>);

// #[derive(Default, Component, Reflect)]
// #[reflect(Component)]
// pub struct InputBuffer(pub Vec<FightInput>);

pub fn buffer_insert_system(
    mut query: Query<&mut InputBuffer, With<Player>>,
    inputs: Res<Vec<(FightInput, InputStatus)>>,
) {
    //println!("Input this frame: {}", inputs[0].0.0);

    for mut buffer in query.iter_mut() {
        buffer.0.push(inputs[0].0)
    }
}


pub fn something_system(
    query: Query<With<Rollback>>
) {
    for _ in query.iter() {
        println!("I don't even know what this is")
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
        ),
        With<Fighter>,
    >,
    state_query: Query<&State>,
    hitbox_query: Query<&HitboxData>
) {
    for (current, map, tf, frame, _buffer) in fighter_query.iter_mut() {
        let state = map.get(&current.0).expect("State doesn't exist.");

        if let Ok(s) = state_query.get(*state) {
            if let Some(hitboxes) = &s.hitboxes {
                if let Some(set) = hitboxes.get(&frame.0) {
                    for h in set {
                        let hitbox = hitbox_query.get(*h).expect("Hitbox entity does not exist");

                        let mut transform = tf.clone();
                        transform.translation += hitbox.offset;

                        commands
                            .entity(*h)
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