use std::fmt::Debug;

use bevy::ecs::reflect;
use bevy::prelude::{Entity, AnimationClip, Handle};
use bevy::reflect::{FromReflect, TypeUuid, Reflect};
use bevy::utils;
use bevy::utils::hashbrown::{HashMap, HashSet};
use bevy::{
    ecs::reflect::ReflectComponent,
    math::Vec3,
    prelude::{Component},
    reflect::{ReflectDeserialize},
};
use bevy_editor_pls::default_windows::inspector::label_button;
use bevy_inspector_egui::{Inspectable, egui};
use serde::de::Visitor;
use serde::{Deserialize, Serialize, de, Deserializer};
use serde_json::from_value;

//use bevy_editor_pls::default_windows::inspector::InspectorWindow;

use crate::input::{NewCommandInput, NewMatchExpression};

use super::modifiers::StateModifier;


#[derive(Default, Debug, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
pub struct StateMap {
    pub map: HashMap<u16, Entity>,
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
    Frame(Option<u16>, Option<u16>),
    // if the fighter just touched the ground
    ReachGround,
    // hitbox id (optional), frame range cancel 
    //OnHit(Option<usize>, u16)

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
    fn get_priority(&self) -> u8;

    fn get_offset(&self) -> Vec3;

    fn set_id(&mut self, value: usize);
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component, Inspectable)]
#[reflect(Component)]
pub struct HitboxData {
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub id: Option<usize>,
    pub radius: f32,
    #[serde(alias = "halfHeight")]
    pub half_height: f32,
    pub offset: Vec3,
    #[serde(default, deserialize_with = "deserialize_rotation")]
    pub rotation: (f32, f32),
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

fn deserialize_rotation<'de, D>(deserializer: D) -> Result<(f32, f32), D::Error>
where
    D: de::Deserializer<'de>, 
{
    struct RotVisitor;

    impl<'de> Visitor<'de> for RotVisitor {
        type Value = (f32, f32);

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a tuple containing a the x and z rotations in radians")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>, 
        {
            let mut x: f32 = seq.next_element()?.expect("Couldn't convert to f32");
            let mut z: f32 = seq.next_element()?.expect("Couldn't convert to f32");

            x = x.to_radians();
            z = z.to_radians();

            Ok((x, z))
            
        }
    }


    deserializer.deserialize_seq(RotVisitor)

}

impl HBox for HitboxData {
    fn get_priority(&self) -> u8 {
        self.priority
    }

    fn get_offset(&self) -> Vec3 {
        self.offset
    }

    fn set_id(&mut self, value: usize) {
        self.id = Some(value);
    }

    
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component, Inspectable)]
#[reflect(Component)]
pub struct HurtboxData {
    #[serde(default)]
    priority: u8,
    #[serde(default)]
    pub id: Option<usize>,
    pub radius: f32,
    #[serde(alias = "halfHeight")]
    pub half_height: f32,
    #[serde(default, deserialize_with = "deserialize_rotation")]
    pub rotation: (f32, f32),
    #[serde(default)]
    pub offset: Vec3,
    #[serde(default, alias = "startFrame")]
    pub start_frame: Option<u16>,
    #[serde(default, alias = "endFrame")]
    pub end_frame: Option<u16>,
}

impl HBox for HurtboxData {
    fn get_priority(&self) -> u8 {
        self.priority
    }

    fn get_offset(&self) -> Vec3 {
        self.offset
    }

    fn set_id(&mut self, value: usize) {
        self.id = Some(value);
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component, Inspectable)]
#[reflect(Component)]
pub struct ProjectileData {
    pub name: String,
    #[serde(alias = "startPosition", default)]
    pub start_position: Vec3, 
    pub dimensions: Vec3,
    #[serde(alias = "velocity", default)]
    pub start_velocity: Vec3,
    #[serde(default)]
    pub acceleration: Vec3,
    #[serde(alias = "spawnFrame")]
    pub spawn_frame: u16,
    #[serde(alias = "lifeFrames")]
    pub life_frames: u16,
    #[serde(default)]
    pub damage: u16,
    #[serde(default = "ProjectileData::max_default")]
    pub max: usize

}

impl ProjectileData {
    fn max_default() -> usize {
        1
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component, Inspectable)]
#[reflect(Component)]
pub struct Velocity(pub Vec3);

#[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component)]
#[reflect(Component)]
pub struct ProjectileReference {
    // projectile name, vec of projectile entities and whether it's in use
    pub projectile_ids: HashMap<String, Vec<(Entity, bool)>>,
    // projectile name, vec of projectiles entities
    //pub projectile_ids: HashMap<String, Vec<Entity>>
    pub amount_in_use: HashMap<String, usize>
}

// impl Inspectable for ProjectileReference {
//     type Attributes = (
//         <String as Inspectable>::Attributes, 
//         <Vec<(Option<Entity>, bool)> as Inspectable>::Attributes
//     );

//     fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: Self::Attributes, context: &mut bevy_inspector_egui::Context) -> bool {
//         let mut changed = false;

//         ui.vertical(|ui| {
//             let mut to_delete = None;
//             let mut to_update = Vec::new();

//             let len = self.projectile_ids.len();
//             for (i, (key, val)) in self.projectile_ids.iter_mut().enumerate() {
//                 let val: Vec<(Option<&mut Entity>, &mut bool)> = val.iter_mut().map(|(entity, b)| {
//                     (Some(entity), b)
//                 }).collect();
                


//                 ui.horizontal(|ui| {
//                     if label_button(ui, "✖", egui::Color32::RED) {
//                         to_delete = Some(key.clone());
//                     }

//                     let mut k = key.clone();
//                     if k.ui(ui, options.0.clone(), &mut context.with_id(i as u64)) {
//                         to_update.push((key.clone(), k));
//                     }

//                     changed |= val.ui(ui, options.1.clone(), &mut context.with_id(i as u64));
//                 });

    //             if i != len - 1 {
    //                 ui.separator();
    //             }
    //         }

    //         ui.vertical_centered_justified(|ui| {
    //             if ui.button("+").clicked() {
    //                 self.projectile_ids.insert(String::default(), vec![(Entity::from_raw(0), false)]);
    //                 changed = true;
    //             }
    //         });

    //         for (old_key, new_key) in to_update.drain(..) {
    //             if let Some(val) = self.projectile_ids.remove(&old_key) {
    //                 self.projectile_ids.insert(new_key, val);
    //                 changed = true;
    //             }
    //         }

    //         if let Some(key) = to_delete {
    //             if self.projectile_ids.remove(&key).is_some() {
    //                 changed = true;
    //             }
    //         }
    //     });

    //     changed

    // }
//}



impl ProjectileReference {
    pub fn new() -> Self {
        Self {
            projectile_ids: HashMap::new(),
            amount_in_use: HashMap::new()
        }
    }

    pub fn insert_ids(&mut self, name: String, ids: Vec<(Entity, bool)>) {
        self.projectile_ids.insert(name, ids);
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


#[derive(Serialize, Deserialize, Default, Debug, Component, Reflect, Clone, Inspectable, Copy, PartialEq)]
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


impl From<f32> for Direction {
    fn from(value: f32) -> Self {
        if value.is_sign_negative() {
            Direction::Left
        }
        else if value.is_sign_positive() {
            Direction::Right
        }
        else {
            panic!()
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


#[derive(Serialize, Deserialize, Default, Debug, Component, Reflect, Clone, Inspectable)]
#[reflect(Component)]
pub struct PlayerAxis {
    pub opponent_pos: Vec3,
    pub x: Vec3,
    pub z: Vec3
}

#[derive(Component)]
pub struct Animation(pub Handle<AnimationClip>, pub f32);

impl Animation {
    pub fn length(&self) -> f32 {
        self.1
    }
}

