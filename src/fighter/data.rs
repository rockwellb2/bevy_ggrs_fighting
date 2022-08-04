use bevy::{prelude::Component, reflect::Reflect};
use bevy::ecs::reflect::ReflectComponent;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct FighterData {
    pub name: String,
    pub walk_speed: f32
}