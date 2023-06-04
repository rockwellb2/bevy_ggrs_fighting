use crate::input::CommandInput;
use bevy::prelude::{Vec3, default};
use bevy::{reflect::Reflect, prelude::Component};
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use bevy::reflect::{reflect_trait, ReflectDeserialize, FromReflect};
use bevy::ecs::reflect::ReflectComponent;

use super::state::{ProjectileData, FrameWindow, Frame};

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
pub struct InputTransition(pub Vec<(CommandInput, u16)>);

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

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct OnExitSetPos {
    // Bone to use as reference
    pub bone: String,
}

#[typetag::serde]
impl StateModifier for OnExitSetPos {
    fn dyn_clone(&self) -> Box<dyn StateModifier> {
        Box::new(self.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct OnExitZeroVelo;

#[typetag::serde]
impl StateModifier for OnExitZeroVelo {
    fn dyn_clone(&self) -> Box<dyn StateModifier> {
        Box::new(self.clone())
    }
}

// The below modifer sets this component
#[derive(Component, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct InputMet(pub bool);

// Command Input and frame window for checking that input
// will attach component to state entity that has a bool whether to transition or not
#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct InputWindowCheck {
    pub window: FrameWindow,
    #[serde(alias = "commandInput", default)]
    pub command_input: CommandInput

}

#[typetag::serde]
impl StateModifier for InputWindowCheck {
    fn dyn_clone(&self) -> Box<dyn StateModifier> {
        Box::new(self.clone())
    }
}






#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct Velo {
    #[serde(alias = "startVelocity", default)]
    pub start_velocity: Option<VectorType>,
    #[serde(default)]
    pub acceleration: Option<VectorType>,

}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component, Clone, FromReflect)]
#[serde(untagged)]
pub enum VectorType {
    Vec(Vec3),
    Variable(String),
    #[default]
    Warning
}

#[typetag::serde]
impl StateModifier for Velo {
    fn dyn_clone(&self) -> Box<dyn StateModifier>  {
        Box::new(self.clone())
    }
}