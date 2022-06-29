use bevy::prelude::*;
use bevy_ggrs::GGRSPlugin;
use ggrs::Config;
use ruwren::{VM, VMConfig, send_foreign, ModuleLibrary, FunctionSignature};
use crate::{fighter::state::{State as FightState, state_system, setup_wren_vm}, util::scripting::{bullshit, Vec3Wrap}, input::{LEFT, Input as FightInput}};

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
    //let lib = bullshit();
    //let vm = VMConfig::new().library(&lib).build();
    //vm.interpret("main", include_str!("../assets/scripts/fighter.wren")).unwrap();
    // vm.execute(|vm| {
    //      vm.ensure_slots(2);
    //      send_foreign!(&vm, "main", "Vec3", Vec3Wrap(Vec3::new(1., 2., 3.)) => 0);
    // });

    let lib = bullshit();
    let vm = VMConfig::new().library(&lib).build();
    vm.interpret("fighter", include_str!("../assets/scripts/fighter.wren")).unwrap();
    vm.interpret("main", include_str!("../assets/data/fighters/tahu/light.wren")).unwrap();


    let handle = vm.make_call_handle(FunctionSignature::new_function("processState", 2));
    vm.execute(|vm| {
        vm.ensure_slots(3);
        vm.set_slot_new_foreign("fighter", "Vec3", Vec3Wrap(Vec3::new(4., 3., 2.)), 1).unwrap();
        vm.set_slot_new_foreign("fighter", "Input", FightInput(LEFT), 2).unwrap();
        vm.get_variable("main", "S100", 0);
    }); 

    //vm.call(FunctionSignature::new_function("getModuleVariables", 1)).unwrap();

    vm.call_handle(&handle).unwrap();
    let position = vm.execute(|vm| {
        vm.ensure_slots(2);
        *vm.get_slot_foreign::<Vec3Wrap>(1).unwrap()
    }).0;

    println!("The vector received from Wren is {}", position);

    //Vec3Wrap::create(&vm, Vec3::new(1., 2., 3.));
    //vm.interpret("test", include_str!("../assets/scripts/test.wren")).unwrap();

    

    let pos = Vec3::new(1., 2., 3.);
    let serialized = serde_json::to_string(&pos).unwrap();

    println!("Serialized vec3: {}", serialized);
    
   

    let deserialized: Vec<FightState> = serde_json::from_str(include_str!("../assets/data/test.json")).unwrap();
    println!("Deserialized file: {:?}", deserialized);

    let mut app = App::new();

    GGRSPlugin::<GGRSConfig>::new()
        .with_update_frequency(FPS)
        .with_input_system(input::input)
        .register_rollback_type::<Transform>()
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
        .add_startup_system(setup_wren_vm.exclusive_system())
        .insert_resource(Msaa { samples: 4 });

    app.run();

}