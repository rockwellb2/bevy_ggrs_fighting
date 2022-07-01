use bevy::{prelude::{Commands, default, Entity}, transform::TransformBundle};

use crate::{fighter::{Fighter, state::CurrentState}, Player};

pub fn spawn_fighters(
    mut commands: Commands,
) 
{
    let player1 = commands
        .spawn_bundle(TransformBundle {
            ..default()
        })
        .insert(Fighter)
        .insert(CurrentState(100))
        .insert(Player(1))
        .id();

    let player2 = commands
        .spawn_bundle(TransformBundle {
            ..default()
        })
        .insert(Fighter)
        .insert(CurrentState(100))
        .insert(Player(2))
        .id();


        let mut load_states = |entity: Entity| {
            commands.entity(entity);

        };

        load_states(player1);
        load_states(player2)

}