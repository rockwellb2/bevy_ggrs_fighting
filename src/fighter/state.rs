use std::any::Any;
use std::fmt::Debug;
use std::process::Output;
use std::{borrow::Borrow, rc::Rc};

use bevy::prelude::Entity;
use bevy::reflect::{reflect_trait, FromReflect};
use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec3,
    prelude::{Component, Query, Transform, With},
    reflect::{Reflect, ReflectDeserialize},
    utils::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize, de};

//use bevy_editor_pls::default_windows::inspector::InspectorWindow;

use crate::input::CommandInput;

use super::Fighter;
use super::systems::InputBuffer;

#[derive(Default, Debug, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct StateMap {
    map: HashMap<u16, Entity>,
}

impl StateMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_state(&mut self, id: u16, state: Entity) {
        self.map.insert(id, state);
    }

    pub fn get<'a>(&'a self, key: &u16) -> Option<&Entity> {
        self.map.get(key)
    }
}

#[typetag::serde]
#[reflect_trait]
pub trait StateModifier: Sync + Send + 'static + Debug + Reflect {}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct Movement;

#[typetag::serde]
impl StateModifier for Movement {}

#[derive(Serialize, Deserialize, Debug, Default, Reflect, Component)]
#[reflect(Component, Deserialize, StateModifier)]
pub struct InputTransition(pub Vec<(CommandInput, u16)>);
    // #[serde(deserialize_with = "deserialize_bits")]
    // bits: u16
    /*
    {
                "InputTransition": {
                    "bits": "0b0"
                }
            }
    
    */


#[typetag::serde]
impl StateModifier for InputTransition {}


#[derive(Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct State {
    pub id: u16,
    duration: Option<u16>,
    hitboxes: Option<HashMap<u16, HashSet<Entity>>>,
}

impl State {
    pub fn from_serialized(serialized: SerializedState) -> Self {
        State {
            id: serialized.id,
            duration: serialized.duration,
            hitboxes: None,
        }
    }

    pub fn add_hitboxes(&mut self, hitboxes: HashMap<u16, HashSet<Entity>>) {
        self.hitboxes = Some(hitboxes);
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct SerializedState {
    pub id: u16,
    #[serde(default)]
    pub debug_name: Option<String>,
    #[serde(default)]
    duration: Option<u16>,
    #[serde(default, alias = "hitboxes")]
    pub unsorted_hitboxes: Option<Vec<HitboxData>>,
    #[serde(default)]
    pub modifiers: Option<Vec<Box<dyn StateModifier>>>,

    //something: Option<dyn Fn<(), Output = ()>>
}

impl SerializedState {
    pub fn new(id: u16, damage: u8, script: Option<String>) -> Self {
        Self {
            id,
            duration: None,
            debug_name: None,
            unsorted_hitboxes: None,
            modifiers: None,
            
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component)]
#[reflect(Component)]
pub struct HitboxData {
    #[serde(default)]
    id: u8,
    dimensions: Vec3,
    offset: Vec3,
    damage: u16,
    #[serde(alias = "startFrame")]
    pub start_frame: u16,
    #[serde(alias = "endFrame")]
    end_frame: u16,
    #[serde(default)]
    rehit: Option<u16> // Number frames after hitting that hitbox can hit again
}

impl HitboxData {
    fn new(id: u8, dimensions: Vec3, offset: Vec3, start_frame: u16, end_frame: u16, damage: u16) -> Self {
        Self {
            id, 
            dimensions,
            offset,
            damage,
            start_frame,
            end_frame,
            rehit: None
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct Variables(HashMap<String, u32>);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CurrentState(pub u16);

impl Default for CurrentState {
    fn default() -> Self {
        Self(1)
    }
}
#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct StateFrame(pub u16);

pub fn state_system(
    mut query: Query<(&mut CurrentState, &StateMap, &mut Transform, &InputBuffer), With<Fighter>>,
    state_query: Query<&State>
) {
    for (current, map, tf, buffer) in query.iter_mut() {
        //let state = map.get(&current.0).expect("State doesn't exist");
        // println!("Current state: {}", current.0);
        // println!("Most recent input: {}", buffer.0.last().unwrap().0)
    }
}
