use bevy::{prelude::*, reflect::{TypeRegistryInternal, TypeRegistry, FromType}};
use bevy_ggrs::GGRSPlugin;
use fighter::state::{CurrentState, Variables, JumpCancel, StateModifier};
use ggrs::Config;



use std::{mem::{size_of, self}, any::Any};

use crate::{fighter::state::{State as FightState, state_system, ReflectStateModifier}, input::{LEFT, Input as FightInput}};

mod fighter;
mod input;
mod battle;
mod util;

const FPS: usize = 60;
const ROLLBACK_DEFAULT: &str = "rollback_default";

#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = input::Input;
    type State = u8;
    type Address = String;
}


#[derive(Component)]
pub struct Player(u8);

fn main() {
    let mut app = App::new();

    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(input::input)
        .register_rollback_type::<Transform>()
        .register_rollback_type::<CurrentState>()
        .register_rollback_type::<Variables>()
        .with_rollback_schedule(
            Schedule::default().with_stage(
                ROLLBACK_DEFAULT,
                SystemStage::parallel()
                .with_system(state_system)

            )
        )
        .build(&mut app);
    

    app
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup.exclusive_system())
        .register_type::<JumpCancel>()
        .insert_resource(Msaa { samples: 4 });

    app.run();

}

fn startup(world: &mut World ) {
    let deserialized: Vec<FightState> = serde_json::from_str(include_str!("../assets/data/fighters/tahu/states.sl.json")).unwrap();
    println!("Deserialized file: {:?}", deserialized);

    let modifier= deserialized.get(0).unwrap().modifiers.as_ref().unwrap().get(0).unwrap();
    //let modifier: &Box<dyn Reflect> = &*modifier.any().downcast_ref::<Box<dyn Reflect>>().unwrap();
    let modifier: Box<dyn Reflect> = modifier.clone_value();
    
    let type_registry = world.get_resource::<TypeRegistry>().unwrap().clone();
    let type_registry = type_registry.read();

    // let reflect_thing = type_registry
    //     .get_type_data::<ReflectStateModifier>(modifier.type_id())
    //     .unwrap();

    // Needs check if entity already has component of type_id()

    let registration = type_registry.get_with_name(modifier.type_name()).unwrap();
    //let reflect_state = type_registry.get_type_data::<ReflectStateModifier>(modifier.type_id()).unwrap();
    let reflect_component = registration.data::<ReflectComponent>().unwrap();


    //let reflect_component = registration.data::<ReflectComponent>().unwrap();
    let entity = world.spawn().id();

    //let something: &Box<dyn Reflect> = modifier.any().downcast_ref::<Box<dyn Reflect>>().unwrap();

    reflect_component.add_component(world, entity,&**&modifier);
    
    //command.spawn().insert(reflect_thing);
}