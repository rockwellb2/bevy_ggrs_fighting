

use bevy::prelude::{Entity, Vec3};

use bevy::reflect::{TypeUuid, Struct};
use bevy::{prelude::Component, reflect::Reflect};
use bevy::ecs::reflect::ReflectComponent;

use parry3d::shape::{Capsule, Segment};
use serde::{Deserialize, Serialize};

use super::hit::components::HitboxData;
use super::state::HurtboxData;

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

#[derive(Component, Clone, Reflect)]
pub struct Collider {
    pub radius: f32,
    pub segment: SegmentProxy
}

impl Default for Collider {
    fn default() -> Self {
        let shape = Capsule::new_y(1., 1.);
        shape.into()
    }
}

impl From<Capsule> for Collider {
    fn from(value: Capsule) -> Self {
        let radius = value.radius;
        let segment = value.segment.into();

        Collider { segment, radius }
    }
}

impl From<&Collider> for Capsule {
    fn from(value: &Collider) -> Self {
        let radius = value.radius;
        let segment = value.segment.clone().into();

        Capsule { segment, radius }
    }
}




#[derive(Clone, Reflect)]
pub struct SegmentProxy {
    pub a: Vec3,
    pub b: Vec3,
}

impl SegmentProxy {
    pub fn from_segment(segment: Segment) -> SegmentProxy {
        segment.into()
    }
}


impl From<SegmentProxy> for Segment {
    fn from(value: SegmentProxy) -> Self {
        Self { a: value.a.into(), b: value.b.into() }
    }
}

impl From<Segment> for SegmentProxy {
    fn from(value: Segment) -> Self {
        Self { a: value.a.into(), b: value.b.into() }
    }
}

   

pub struct CollisionData {
    pub attacker_box: HitboxData,
    pub attacker: Entity,
    pub recipient_box: HurtboxData,
    pub recipient: Entity,
}

impl CollisionData {
    pub fn get_attacker_priority(&self) -> u8 {
        self.attacker_box.priority
    }

    pub fn get_recipient(&self) -> Entity {
        self.recipient
    }
}

pub struct HitEvent(pub CollisionData);