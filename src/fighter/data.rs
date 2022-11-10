use std::any;

use bevy::prelude::{Entity, Vec3};
use bevy::reflect::utility::GenericTypeInfoCell;
use bevy::reflect::{TypeUuid, Typed, TypeInfo, StructInfo, NamedField, Struct, GetTypeRegistration, TypeRegistration, DynamicStruct, FieldIter};
use bevy::{prelude::Component, reflect::Reflect};
use bevy::ecs::reflect::ReflectComponent;
use bevy_inspector_egui::Inspectable;
use parry3d::shape::{Cuboid, Capsule, Segment};
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

#[derive(Component, Clone, Reflect, Inspectable)]
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

impl Into<Capsule> for Collider {
    fn into(self) -> Capsule {
        let radius = self.radius.into();
        let segment = self.segment.into();

        Capsule {
            segment,
            radius,
        }

    }
}

impl Into<Capsule> for &Collider {
    fn into(self) -> Capsule {
        let radius = self.radius.into();
        let segment = self.segment.clone().into();

        Capsule {
            segment,
            radius,
        }
    }
}

impl Into<Collider> for Capsule {
    fn into(self) -> Collider {
        let radius = self.radius.into();
        let segment = self.segment.into();

        Collider {
            radius,
            segment,
        }
    }
}



#[derive(Clone, Reflect, Inspectable)]
pub struct SegmentProxy {
    pub a: Vec3,
    pub b: Vec3,
}

impl SegmentProxy {
    pub fn from_segment(segment: Segment) -> SegmentProxy {
        segment.into()
    }
}


impl Into<SegmentProxy> for Segment {
    fn into(self) -> SegmentProxy {
        SegmentProxy { a: self.a.into(), b: self.b.into() }
    }
}

impl Into<Segment> for SegmentProxy {
    fn into(self) -> Segment {
        Segment { a: self.a.into(), b: self.b.into() }
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

