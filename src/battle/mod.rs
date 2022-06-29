use bevy::{prelude::{Commands, default}, transform::TransformBundle};

use crate::{fighter::{Fighter, state::CurrentState}, Player};

pub fn spawn_fighters(
    mut commands: Commands,
) 
{
    commands
        .spawn_bundle(TransformBundle {
            ..default()
        })
        .insert(Fighter)
        .insert(CurrentState(100))
        .insert(Player(1));

    commands
        .spawn_bundle(TransformBundle {
            ..default()
        })
        .insert(Fighter)
        .insert(CurrentState(100))
        .insert(Player(2));

}