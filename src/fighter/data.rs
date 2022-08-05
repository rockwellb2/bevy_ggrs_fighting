use bevy::reflect::TypeUuid;
use bevy::{prelude::Component, reflect::Reflect};
use bevy::ecs::reflect::ReflectComponent;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Component, Reflect, TypeUuid)]
#[reflect(Component)]
#[uuid = "d99f5e90-13a4-11ed-861d-0242ac120002"]
pub struct FighterData {
    pub name: String,
    pub walk_speed: f32
}