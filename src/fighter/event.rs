use bevy::prelude::Entity;

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