use bevy::{
    core::Name,
    math::Vec2,
    prelude::{default, Color, Commands, Entity, ResMut, OrthographicCameraBundle, Res},
    sprite::{Sprite, SpriteBundle}
};
use bevy_ggrs::{Rollback, RollbackIdProvider};
use ggrs::{P2PSession, SyncTestSession};

use crate::{
    fighter::{data::FighterData, state::{CurrentState, StateFrame}, Fighter, systems::InputBuffer},
    Player, GGRSConfig, input::{Input as FightInput, BUFFER_SIZE}, util::Buffer,
};

//#[derive(Default)]
pub struct PlayerEntities(pub Entity, pub Entity);

impl PlayerEntities {
    pub fn get(&self, n: u8) -> Entity {
        match n {
            1 => self.0,
            2 => self.1,
            _ => panic!("Player number {} doesn't exist!", n),
        }
    }
}

pub fn spawn_fighters(
    mut commands: Commands, 
    mut rip: ResMut<RollbackIdProvider>,
    sync_test_session: Option<Res<SyncTestSession<GGRSConfig>>>
) {
    let num_players = sync_test_session
        .map(|s| s.num_players())
        .expect("Couldn't find Session");






    let fighter1: FighterData = serde_json::from_str(include_str!(
        "../../assets/data/fighters/tahu/fighter_data.json"
    ))
    .unwrap();
    let fighter2: FighterData = serde_json::from_str(include_str!(
        "../../assets/data/fighters/abe/fighter_data.json"
    ))
    .unwrap();

    let player1 = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Player 1"))
        .insert(Fighter)
        .insert(fighter1)
        .insert(CurrentState(0))
        .insert(Player(1))
        .insert(Rollback::new(rip.next_id()))
        .insert(StateFrame(0))
        //.insert(InputBuffer(vec![FightInput::default(); BUFFER_SIZE]))
        .insert(InputBuffer(Buffer::with_capacity(BUFFER_SIZE)))
        .id();

    let player2 = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Player 2"))
        .insert(Fighter)
        .insert(fighter2)
        .insert(CurrentState(0))
        .insert(Player(2))
        .insert(Rollback::new(rip.next_id()))
        .insert(StateFrame(0))
        .id();

    commands.insert_resource(PlayerEntities(player1, player2));


    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);

    // let mut load_states = |entity: Entity| {
    //     commands.entity(entity);

    // };

    // load_states(player1);
    // load_states(player2)
}
