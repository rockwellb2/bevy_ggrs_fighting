



use super::{
    data::FighterData,
    state::{CurrentState, Movement, StateFrame, StateMap, State, InputTransition, HitboxData, Active, Owner, Direction, AdjustFacing, Facing},
    Fighter,
};
use bevy::{
    ecs::reflect::ReflectComponent,
    prelude::{Component, Entity, Query, Res, Transform, With, Commands, SpatialBundle, Visibility},
    reflect::{Reflect},
    utils::default
};
use bevy_ggrs::Rollback;
use ggrs::InputStatus;

use crate::{
    input::{Input as FightInput, RAW_LEFT, RAW_RIGHT, RAW_LP, StateInput, ButtonPress, LEFT_HELD, LEFT, MAP, DirectionalInput, RIGHT, RIGHT_HELD, RAW_DOWN, RAW_UP, DOWN, DOWN_HELD, UP, UP_HELD}, Player, FPS, util::Buffer, battle::PlayerEntities,
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

                if unpacked.x == DirectionalInput::None {
                    current.0 = 0;
                    return
                }

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
                for (command, to_state) in transitions.0.iter() {
                    if command.compare(&buffer.0) {
                        current.0 = *to_state;
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
pub struct InputBuffer(pub Buffer<u32>);
//pub struct InputBuffer(pub Buffer<FightInput>);

// #[derive(Default, Component, Reflect)]
// #[reflect(Component)]
// pub struct InputBuffer(pub Vec<FightInput>);

pub fn buffer_insert_system(
    mut query: Query<&mut InputBuffer, With<Player>>,
    inputs: Res<Vec<(FightInput, InputStatus)>>,
) {
    //println!("Input this frame: {}", inputs[0].0.0);

    for mut buffer in query.iter_mut() {
        // let previous: u32 = *buffer.0.get(0).unwrap_or(&0);
        // let raw: u32 = inputs[0].0.0;


        // let prev_compare = |i: (u16, u32, u32)| -> ButtonPress {
        //     let check = previous & i.2 == i.2 || previous & i.1 == i.1;

        //     return if raw & i.0 != 0 {
        //         if check { ButtonPress::Hold} else { ButtonPress::Press }
        //     }
        //     else {
        //         if check { ButtonPress::Release } else { ButtonPress::None }
        //     }
        // };
        
        // let lp = prev_compare(MAP[0]);
        // let mp = prev_compare(MAP[1]);
        // let hp = prev_compare(MAP[2]);
        // let lk = prev_compare(MAP[3]);
        // let mk = prev_compare(MAP[4]);
        // let hk = prev_compare(MAP[5]);
        
        
        // let x = if raw & RAW_LEFT != 0 {
        //     DirectionalInput::Negative
        // }
        // else if raw & RAW_RIGHT != 0 {
        //     DirectionalInput::Positive
        // }
        // else {
        //     DirectionalInput::None
        // };

        // let just_pressed_x = match x {
        //     DirectionalInput::None => false,
        //     DirectionalInput::Positive => previous & LEFT == LEFT || previous & LEFT_HELD == LEFT_HELD,
        //     DirectionalInput::Negative => previous & RIGHT == RIGHT || previous & RIGHT_HELD == RIGHT_HELD,
        // };

        // let y = if raw & RAW_DOWN != 0 {
        //     DirectionalInput::Negative
        // }
        // else if raw & RAW_UP != 0 {
        //     DirectionalInput::Positive
        // }
        // else {
        //     DirectionalInput::None
        // };

        // let just_pressed_y = match x {
        //     DirectionalInput::None => false,
        //     DirectionalInput::Positive => previous & DOWN == DOWN || previous & DOWN_HELD == DOWN_HELD,
        //     DirectionalInput::Negative => previous & UP == UP || previous & UP_HELD == UP_HELD,
        // };

        // let input: u32 = StateInput::new(lp, mp, hp, lk, mk, hk, x, just_pressed_x, y, just_pressed_y).into();
        // buffer.0.push(input);

        buffer.0.push(inputs[0].0.0)
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
            &Facing
        ),
        With<Fighter>,
    >,
    state_query: Query<&State>,
    hitbox_query: Query<(&HitboxData)>
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
                        //println!("Offset.x: {}", offset.x);

                        let mut transform = tf.clone();
                        transform.translation += offset;

                        //println!("Transform: {:?}", transform.translation);

                        commands
                            .entity(*h)
                            .insert(Active)
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
    fighter_query: Query<&StateFrame, With<Fighter>>
) {
    for (entity, data, owner, mut visible) in query.iter_mut() {
        let frame = fighter_query.get(owner.0).expect("Owner doesn't exist");

        if frame.0 > data.end_frame {
            visible.is_visible = false;
            commands.entity(entity).remove::<Active>();
        }

    }

}


pub fn adjust_facing_system(
    players: Res<PlayerEntities>,
    mut fighter_query: Query<(&CurrentState, &StateMap, &Transform, &mut Facing), With<Fighter>>,
    state_query: Query<With<AdjustFacing>, With<State>>
) {
    let player1 = players.get(1);
    let player2 = players.get(2);

    if let Ok([
        (current1, map1, tf1, mut facing1), 
        (current2, map2, tf2, mut facing2)
    ]) = fighter_query.get_many_mut([player1, player2]) 
    {
        let state1 = map1.get(&current1.0).unwrap();
        let state2 = map2.get(&current2.0).unwrap();

        if let Ok(_) = state_query.get(*state1) {
            facing1.0 = if tf1.translation.x > tf2.translation.x {
                Direction::Left
            }
            else {
                Direction::Right
            };
        }

        if let Ok(_) = state_query.get(*state2) {
            facing2.0 = if tf1.translation.x > tf2.translation.x {
                Direction::Right
            }
            else {
                Direction::Left
            }
        }        
    }


}