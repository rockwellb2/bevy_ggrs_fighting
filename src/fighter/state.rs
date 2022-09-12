use std::fmt::Debug;

use bevy::prelude::Entity;
use bevy::reflect::{reflect_trait, FromReflect, TypeUuid};
use bevy::utils::hashbrown::{HashMap, HashSet};
use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec3,
    prelude::{Component},
    reflect::{Reflect, ReflectDeserialize},
};
use bevy_inspector_egui::Inspectable;
use serde::de::Visitor;
use serde::{Deserialize, Serialize, de, Deserializer};
use serde_json::from_value;

//use bevy_editor_pls::default_windows::inspector::InspectorWindow;

use crate::input::{NewCommandInput, NewMatchExpression};


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


#[derive(Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct State {
    pub id: u16,
    pub duration: Option<u16>,
    pub hitboxes: Option<HashMap<u16, HashSet<Entity>>>,
    pub hurtboxes: Option<HashMap<u16, HashSet<Entity>>>,
    pub transitions: Vec<Entity>,
    pub triggers: (Option<Vec<Conditions>>, Vec<Vec<Conditions>>)
}

impl State {
    pub fn from_serialized(serialized: SerializedState) -> Self {
        State {
            id: serialized.id,
            duration: serialized.duration,
            hitboxes: None,
            hurtboxes: None,
            transitions: Vec::new(),
            triggers: serialized.triggers
        }
    }

    pub fn add_hitboxes(&mut self, hitboxes: HashMap<u16, HashSet<Entity>>) {
        self.hitboxes = Some(hitboxes);
    }

    pub fn add_hurtboxes(&mut self, hurtboxes: HashMap<u16, HashSet<Entity>>) {
        self.hurtboxes = Some(hurtboxes);
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, FromReflect, Reflect)]
#[serde(rename_all = "camelCase")]
pub enum Conditions {
    // used for the current state
    In(Vec<u16>),
    NotIn(u16),
    Command(NewCommandInput),
    // when current state is at the end of its duration
    EndDuration,
    // current frame of the stat
    Frame(Option<u16>, Option<u16>)
}

#[derive(Default, Serialize, Debug, Clone)]
pub struct SerializedState {
    pub id: u16,
    pub debug_name: Option<String>,
    duration: Option<u16>,
    pub unsorted_hitboxes: Option<Vec<HitboxData>>,
    pub unsorted_hurtboxes: Option<Vec<HurtboxData>>,
    pub modifiers: Option<Vec<Box<dyn StateModifier>>>,
    pub transitions: Vec<u16>,
    pub triggers: (Option<Vec<Conditions>>, Vec<Vec<Conditions>>)
}


impl<'de> Deserialize<'de> for SerializedState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let json: serde_json::value::Value = serde_json::value::Value::deserialize(deserializer)?;
        let object = json.as_object().expect("Not an object");

        let mut id: u16 = 0;
        let mut debug_name: Option<String> = None;
        let mut duration: Option<u16> = None;
        let mut unsorted_hitboxes: Option<Vec<HitboxData>> = None;
        let mut unsorted_hurtboxes: Option<Vec<HurtboxData>> = None;
        let mut modifiers: Option<Vec<Box<dyn StateModifier>>> = None;
        let mut transitions: Vec<u16> = vec![0];
        let mut triggers: (Option<Vec<Conditions>>, Vec<Vec<Conditions>>) = (None, Vec::new());

        for (key, value) in object.into_iter() {
            let key = key.as_str();

            if key == "id" {
                id = value.as_u64().expect("u64") as u16;
            }
            else if key == "debug_name" {
                debug_name = Some(value.as_str().expect("str").to_string());
            }
            else if key == "duration" {
                duration = Some(value.as_u64().expect("u64") as u16);
            }
            else if key == "hitboxes" {
                unsorted_hitboxes = Some(from_value(value.clone()).expect("Can't convert array to Vec<HitboxData>"));
            }
            else if key == "hurtboxes" {
                unsorted_hurtboxes = Some(from_value(value.clone()).expect("Can't convert array to Vec<HurtboxData>"));
            }

            else if key == "modifiers" {
                modifiers = Some(from_value(value.clone()).expect("Can't convert array to Vec<Box<dyn StateModifier>>"));
            }

            else if key == "transitions" {
                transitions = from_value(value.clone()).expect("Can't convert array to Vec<u16>");
            }

            else if key == "triggerAll" {
                triggers.0 = Some(from_value(value.clone()).expect("Can't convert array to Vec<Conditions>"));
            }

            else if key.contains("trigger") {
                triggers.1.push(from_value(value.clone()).expect("Can't convert array to Vec<Conditions>"))
            }
        }

        Ok(SerializedState {
            id,
            debug_name,
            duration,
            unsorted_hitboxes,
            unsorted_hurtboxes,
            modifiers,
            transitions,
            triggers,
        })

    }
}


#[derive(Serialize, Deserialize, TypeUuid, Clone)]
#[uuid = "57ae9bea-139e-11ed-861d-0242ac120002"]
pub struct SerializedStateVec(pub Vec<SerializedState>);

pub trait HBox: Component {
    fn get_id(&self) -> u8;

    fn get_offset(&self) -> Vec3;
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component)]
#[reflect(Component)]
pub struct HitboxData {
    #[serde(default)]
    pub id: u8,
    pub dimensions: Vec3,
    pub offset: Vec3,
    pub damage: u16,
    pub hitstun: u16,
    pub blockstun: u16,
    #[serde(alias = "startFrame")]
    pub start_frame: u16,
    #[serde(alias = "endFrame")]
    pub end_frame: u16,
    #[serde(default)]
    rehit: Option<u16> // Number frames after hitting that hitbox can hit again
}

impl HBox for HitboxData {
    fn get_id(&self) -> u8 {
        self.id
    }

    fn get_offset(&self) -> Vec3 {
        self.offset
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component)]
#[reflect(Component)]
pub struct HurtboxData {
    #[serde(default)]
    id: u8,
    pub dimensions: Vec3,
    #[serde(default)]
    pub offset: Vec3,
    #[serde(default, alias = "startFrame")]
    pub start_frame: Option<u16>,
    #[serde(default, alias = "endFrame")]
    pub end_frame: Option<u16>,
}

impl HBox for HurtboxData {
    fn get_id(&self) -> u8 {
        self.id
    }

    fn get_offset(&self) -> Vec3 {
        self.offset
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

#[derive(Default, Reflect, Component)]
#[component(storage = "SparseSet")]
pub struct Active(pub HashSet<Entity>);
// Ignored Entities


#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct StateFrame(pub u16);

#[derive(Component, Inspectable, PartialEq)]
pub struct Owner(pub Entity);


#[derive(Serialize, Deserialize, Default, Debug, Component, Reflect, Clone, Inspectable, Copy)]
pub enum Direction {
    Left, 
    #[default]
    Right
}

impl Direction {
    pub fn sign(&self) -> f32 {
        match self {
            Direction::Left => -1.,
            Direction::Right => 1.,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Component, Reflect, Clone, Inspectable)]
#[reflect(Component)]
pub struct Facing(pub Direction);

#[derive(Serialize, Deserialize, Default, Debug, Component, Reflect, Clone, Inspectable)]
#[reflect(Component)]
pub struct Health(pub u16);

#[derive(Serialize, Deserialize, Default, Debug, Component, Reflect, Clone, Inspectable)]
#[reflect(Component)]
pub struct InHitstun(pub u16);

