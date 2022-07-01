use bevy::prelude::*;
use bevy_ggrs::GGRSPlugin;
use fighter::state::{CurrentState, Variables};
use ggrs::Config;


use std::mem::{size_of, self};

use crate::{fighter::state::{State as FightState, state_system}, input::{LEFT, Input as FightInput}};

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

    

    //let deserialized: Vec<FightState> = serde_json::from_str(include_str!("../assets/data/test.sl.json")).unwrap();
    //println!("Deserialized file: {:?}", deserialized);

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
        .insert_resource(Msaa { samples: 4 });

    app.run();

}