use crate::input::NewCommandInput;
use bevy::{reflect::Reflect, prelude::Component};
use bevy_inspector_egui::Inspectable;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use bevy::reflect::{reflect_trait, ReflectDeserialize};
use bevy::ecs::reflect::ReflectComponent;

use super::state::{HitboxData, ProjectileData};

#[typetag::serde]
#[reflect_trait]
pub trait StateModifier: Sync + Send + 'static + Debug + Reflect {
    fn dyn_clone(&self) -> Box<dyn StateModifier>;
}

impl Clone for Box<dyn StateModifier> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct Movement;

#[typetag::serde]
impl StateModifier for Movement {
    fn dyn_clone(&self) -> Box<dyn StateModifier> {
        Box::new(self.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct InputTransition(pub Vec<(NewCommandInput, u16)>);

#[typetag::serde]
impl StateModifier for InputTransition {
    fn dyn_clone(&self) -> Box<dyn StateModifier> {
        Box::new(self.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct AdjustFacing;

#[typetag::serde]
impl StateModifier for AdjustFacing {
    fn dyn_clone(&self) -> Box<dyn StateModifier> {
        Box::new(self.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone, Inspectable)]
pub enum Object {
    Projectile(ProjectileData),
    #[default]
    None,
}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct CreateObject(pub Object);

#[typetag::serde]
impl StateModifier for CreateObject {
    fn dyn_clone(&self) -> Box<dyn StateModifier>  {
        Box::new(self.clone())
    }

}