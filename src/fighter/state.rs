use std::{rc::Rc, borrow::Borrow};

use bevy::{utils::HashMap, math::Vec3, prelude::{Component, Query, With, Transform, Handle, Res, World, NonSend}};
use ruwren::{Handle as WrenHandle, VMConfig, FunctionSignature, VMWrapper, FunctionHandle};
use serde::{Serialize, Deserialize};

use super::Fighter;



#[derive(Serialize, Deserialize, Component)]
pub struct StateMap {
    map: HashMap<u8, State>,
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

    pub fn get<'a>(&'a self, key: &u8) -> Option<&'a State> {
        self.map.get(key)
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub id: u8,
    #[serde(skip)]
    debug_name: Option<String>,
    #[serde(default)]
    script: Option<String>,
    #[serde(default)]
    hitboxes: Option<Vec<HitboxData>>,

}

impl State {
    pub fn new(id: u8, script: Option<String>) -> Self {
        Self {
            id,
            script,
            debug_name: None,
            hitboxes: None,
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
        //dimensions.x

        Self { 
            dimensions, 
            offset 
        }
    }
}


#[derive(Component)]
pub struct CurrentState(pub u8);

pub fn state_system(
    mut query: Query<(&mut CurrentState, &StateMap, &mut Transform), With<Fighter>>,
    res: NonSend<WrenVM>
) {
    for (current, map, tf) in query.iter_mut() {
        if let Some(state) = map.get(&current.0) {
            

        }
    }
}


pub struct HandleWrapper(Rc<FunctionHandle<'static>>);

pub struct WrenVM(VMWrapper);


pub fn setup_wren_vm(world: &mut World) {
    let vm = VMConfig::new().build();
    world.insert_non_send_resource(WrenVM(vm));

}

pub fn setup_wren_handle(world: &mut World, vm: NonSend<WrenVM>) {
//     let handle = vm.0.make_call_handle(FunctionSignature::new_function("processState", 2)).clone();
//     world.insert_non_send_resource(handle)

    // let handle = vm.0.make_call_handle(FunctionSignature::new_function("processState", 2));
    // world.insert_non_send_resource(HandleWrapper(handle));



}