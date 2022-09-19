use bevy::prelude::Entity;
use bevy::reflect::TypeUuid;
use bevy::{prelude::Component, reflect::Reflect};
use bevy::ecs::reflect::ReflectComponent;
use parry3d::shape::Cuboid;
use serde::{Deserialize, Serialize};

use super::state::{HitboxData, HurtboxData};

#[derive(Clone, Default, Debug, Serialize, Deserialize, Component, Reflect, TypeUuid)]
#[reflect(Component)]
#[uuid = "d99f5e90-13a4-11ed-861d-0242ac120002"]
pub struct FighterData {
    pub name: String,
    #[serde(alias = "walkSpeed")]
    pub walk_speed: f32,
    #[serde(alias = "walkForward", default)]
    pub walk_forward: f32,
    #[serde(alias = "walkBack", default)]
    pub walk_back: f32
}

#[derive(Component)]
pub struct Collider {
    pub shape: Cuboid
}


pub struct CollisionData {
    pub attacker_box: HitboxData,
    pub attacker: Entity,
    pub recipient_box: HurtboxData,
    pub recipient: Entity,
}

impl CollisionData {
    pub fn get_attacker_id(&self) -> u8 {
        self.attacker_box.id
    }

    pub fn get_recipient(&self) -> Entity {
        self.recipient
    }
}

pub struct HitEvent(pub CollisionData);

