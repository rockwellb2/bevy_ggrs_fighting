

use super::{
    data::FighterData,
    state::{CurrentState, Movement, StateFrame, StateMap, State, InputTransition},
    Fighter,
};
use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Component, Entity, Query, Res, Transform, With},
    reflect::{Reflect},
};
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
        frame.0 = frame.0.checked_add(1).unwrap_or(0);
    }
}

pub fn process_input_system(
    mut query: Query<(&mut CurrentState, &StateMap, &InputBuffer), (With<Fighter>, With<Player>)>,
    state_query: Query<&InputTransition, With<State>>
) {
    for (mut current, map, buffer) in query.iter_mut() {
        //println!("Does this print?");
        
        let state: &Entity = map.get(&current.0).expect("State doesn't exist");

        if let Ok(transitions) = state_query.get(*state) {
            //println!("Always prints after the above");
            for (command, u16) in transitions.0.iter() {
                if command.compare(&buffer.0) {
                    current.0 = *u16;
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
