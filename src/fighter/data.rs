use std::any;

use bevy::prelude::{Entity, Vec3};
use bevy::reflect::utility::GenericTypeInfoCell;
use bevy::reflect::{TypeUuid, Typed, TypeInfo, StructInfo, NamedField, Struct};
use bevy::{prelude::Component, reflect::Reflect};
use bevy::ecs::reflect::ReflectComponent;
use parry3d::shape::{Cuboid, Capsule};
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
    pub shape: Capsule
}

impl Reflect for Collider {
    fn type_name(&self) -> &str {
        any::type_name::<Self>()
    }

    fn get_type_info(&self) -> &'static bevy::reflect::TypeInfo {
        <Self as Typed>::type_info()
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
       self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn apply(&mut self, value: &dyn Reflect) {
        todo!()
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }

    fn reflect_ref(&self) -> bevy::reflect::ReflectRef {
        //bevy::reflect::ReflectRef::Struct(self)
        todo!()
    }

    fn reflect_mut(&mut self) -> bevy::reflect::ReflectMut {
        todo!()
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        todo!()
    }
}

impl Typed for Collider {
    fn type_info() -> &'static TypeInfo {
        static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
        CELL.get_or_insert::<Self, _>(|| TypeInfo::Struct(
            StructInfo::new::<Collider>(&[NamedField::new::<Collider, &str>("segment".into()), NamedField::new::<Collider, &str>("radius".into())])
        ))
    }
}

// impl Struct for Collider {
//     fn field(&self, name: &str) -> Option<&dyn Reflect> {
//         match name {
//             "radius" => Some(&self.shape.radius),
//             "segment" => Some({
//                 let a: Vec3 = self.shape.segment.a.into();
//                 let b: Vec3 = self.shape.segment.b.into();

//                 (a.clone(), b.clone())
//             }),
//             _ => None
//         }
//     }

//     fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
//         todo!()
//     }

//     fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
//         todo!()
//     }

//     fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
//         todo!()
//     }

//     fn name_at(&self, index: usize) -> Option<&str> {
//         todo!()
//     }

//     fn field_len(&self) -> usize {
//         todo!()
//     }

//     fn iter_fields(&self) -> bevy::reflect::FieldIter {
//         todo!()
//     }

//     fn clone_dynamic(&self) -> bevy::reflect::DynamicStruct {
//         todo!()
//     }
// }


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

