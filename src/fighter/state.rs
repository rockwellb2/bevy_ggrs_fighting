use std::{rc::Rc, borrow::Borrow};

use bevy::{utils::{HashMap}, math::Vec3, prelude::{Component, Query, With, Transform, Handle, Res, World, NonSend}, reflect::Reflect};
use serde::{Serialize, Deserialize};

use super::Fighter;



#[derive(Serialize, Deserialize, Component)]
pub struct StateMap {
    map: HashMap<u16, State>,
}

impl StateMap {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    fn add_state(&mut self, state: State) {
        self.map.insert(state.id, state);
    }

    pub fn from_vec_states(states: Vec<State>) -> Self {
        let mut map = StateMap::new();

        for state in states {
            map.add_state(state);
        }

        map
    }

    pub fn get<'a>(&'a self, key: &u16) -> Option<&'a State> {
        self.map.get(key)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StateFlags {
    Movement
    //Movement(Fn(&mut Transform))
}




#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub id: u16,
    #[serde(skip)]
    debug_name: Option<String>,
    #[serde(default)]
    duration: Option<u8>,
    #[serde(default)]
    damage: u8,
    #[serde(default)]
    script: Option<String>,
    #[serde(default, alias = "hitboxes")]
    unsorted_hitboxes: Option<Vec<HitboxData>>,
    #[serde(default)]
    pub flags: Option<Vec<StateFlags>>

}

impl State {
    pub fn new(id: u16, damage: u8, script: Option<String>) -> Self {
        Self {
            id,
            script,
            damage,
            duration: None,
            debug_name: None,
            unsorted_hitboxes: None,
            flags: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HitboxData {
    dimensions: Vec3,
    offset: Vec3,
}

impl HitboxData {
    fn new(dimensions: Vec3, offset: Vec3) -> Self {
        Self { 
            dimensions, 
            offset 
        }
    }
}


#[derive(Component, Reflect, Default)]
pub struct Variables(HashMap<String, u32>);


#[derive(Component, Reflect)]
pub struct CurrentState(pub u16);

impl Default for CurrentState {
    fn default() -> Self {
        Self(1)
    }
}

pub fn state_system(
    mut query: Query<(&mut CurrentState, &StateMap, &mut Transform), With<Fighter>>,
    
) {
    for (current, map, tf) in query.iter_mut() {
        if let Some(state) = map.get(&current.0) {
            // flags
            if let Some(flags) = &state.flags {
                for flag in flags {
                    match flag {
                        StateFlags::Movement => print!("Movement!"),
                    }
                }
            }

            // hitboxes
            

        }
    }
}


