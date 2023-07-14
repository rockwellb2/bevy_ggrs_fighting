use std::marker::PhantomData;

use bevy::prelude::{Entity, EventReader, EventWriter, Component, Query, With};

use super::{modifiers::{StateModifier, OnExitZeroVelo}, state::CurrentState};

pub struct TransitionEvent {
    pub fighter: Entity,
    pub to_id: u16,
}

impl TransitionEvent {
    pub fn new(fighter: Entity, to_id: u16) -> Self {
        Self {
            fighter,
            to_id
        }
    }
}

pub struct StateEvent<T: StateModifier, S: ScheduleTiming> {
    timing: PhantomData<S>,
    modifier: T
}

impl<T: StateModifier, S: ScheduleTiming> StateEvent<T, S> {
    pub fn new(modifier: T) -> Self {
        StateEvent { timing: PhantomData, modifier }
    }
}

pub trait ScheduleTiming: Send + Sync + 'static + Component {}

#[derive(Component)]
pub struct OnEnter;
#[derive(Component)]
pub struct OnExit;
#[derive(Component)]
pub struct OnProcess;

impl ScheduleTiming for OnEnter {}
impl ScheduleTiming for OnExit {}
impl ScheduleTiming for OnProcess {}


pub fn resolve_state_system<S: ScheduleTiming>(
    event_reader: EventReader<StateEvent<OnExitZeroVelo, S>>

) {

}



pub fn state_event_writer_system<T: StateModifier + Component + Clone, S: ScheduleTiming>(
    player_query: Query<&CurrentState>,
    state_query: Query<&ModifierEntityList>,
    list_query: Query<&T, With<S>>,
    mut event_writer: EventWriter<StateEvent<T, S>>
) {

    for current in player_query.iter() {
        if let Ok(list) = state_query.get(current.0) {
            for modifier in list_query.iter_many(&list.0) {
                event_writer.send(StateEvent::new(modifier.clone()));
            }
        }
    }
}

#[derive(Component)]
pub struct ModifierEntityList(pub Vec<Entity>);
