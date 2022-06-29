use bevy::{prelude::Transform, math::Vec3};
use ruwren::{create_module, Class, get_slot_checked, VM, ModuleLibrary, VMConfig, VMWrapper, send_foreign};

use crate::{fighter::Fighter, input::{LEFT, RIGHT, Input as FightInput}};

#[derive(Clone, Copy)]
pub struct Vec3Wrap(pub Vec3);

impl Class for Vec3Wrap {
    // This function is for calling from Wren for initialization 
    fn initialize(vm: &VM) -> Self {
        let x = get_slot_checked!(vm => num 1) as f32;
        let y = get_slot_checked!(vm => num 2) as f32;
        let z = get_slot_checked!(vm => num 3) as f32;

        Vec3Wrap(Vec3::new(x, y, z))
    }
}

impl Vec3Wrap {
    pub fn create(vm_wrapper: &VMWrapper, vector: Vec3) {
        vm_wrapper.execute(|vm| {
            //send_foreign!(&vm, "main", "Vec3", Vec3Wrap(vector) => 0);
            vm.set_slot_new_foreign("main", "Vec3", Vec3Wrap(vector), 0).unwrap();
            //vm.get_slot_foreign::<Vec3Wrap>(0).expect("Immediately under");
        });
    }


    pub fn x(&self, vm: &VM) {
        vm.set_slot_double(0, self.0.x.into());
    }

    fn y(&self, vm: &VM) {
        vm.set_slot_double(0, self.0.y.into());
    }

    fn z(&self, vm: &VM) {
        vm.set_slot_double(0, self.0.z.into());
    }

    pub fn set_x(&mut self, vm: &VM) {
        self.0.x = get_slot_checked!(vm => num 1) as f32;
    }

    fn set_y(&mut self, vm: &VM) {
        self.0.y = get_slot_checked!(vm => num 1) as f32;
    }

    fn set_z(&mut self, vm: &VM) {
        self.0.z = get_slot_checked!(vm => num 1) as f32;
    }

}

struct State;

impl Class for State {
    fn initialize(_: &VM) -> Self
    where
        Self: Sized {
        todo!()
    }
}


create_module! {
    class("Vec3") crate::util::scripting::Vec3Wrap => vec3wrap {
        instance(getter "x") x,
        instance(getter "y") y,
        instance(getter "z") z,
        instance(setter "x") set_x,
        instance(setter "y") set_y,
        instance(setter "z") set_z
    }

    class("Helper") crate::util::scripting::Helper => helper {
        static(fn "fighterMovement", 2) fighter_movement
    }

    class("Input") crate::input::Input => input {

    }

    module =>  fighter
}

pub fn bullshit() -> ModuleLibrary {
    let mut lib = ModuleLibrary::new();
    fighter::publish_module(&mut lib);
    lib
}


pub struct Helper;

impl Class for Helper {
    fn initialize(_: &ruwren::VM) -> Self {
        todo!()
    }
}

impl Helper {
    pub fn fighter_movement(vm: &VM) {
        vm.ensure_slots(2);
        let pos = vm.get_slot_foreign_mut::<Vec3Wrap>(1).unwrap();
        let input = vm.get_slot_foreign::<FightInput>(2).unwrap();

        if input.0 & LEFT != 0 {
            //transform.translation.x -= 5.;
            pos.0.x -= 5.;
        }
         if input.0 & RIGHT != 0 {
            //transform.translation.x += 5.;
            pos.0.x += 5.;
        }

        // let send = vm.set_slot_new_foreign("fighter", "Vec3", pos.to_owned(), 0);
        // if send.is_err() {
        //     panic!("Could not send Vec3 object");
        // }
    }
}