use bevy::{
    core::Name,
    math::Vec2,
    prelude::{default, Color, Commands, Entity, ResMut, Res, AssetServer, Handle, Assets, Camera2dBundle, OrthographicProjection, Visibility, Transform, Vec3, KeyCode},
    sprite::{Sprite, SpriteBundle}
};

use bevy_ggrs::Rollback;
use ggrs::{SyncTestSession};

use iyes_progress::prelude::AssetsLoading;
use leafwing_input_manager::{InputManagerBundle, prelude::{ActionState, InputMap}};


use crate::{
    fighter::{data::FighterData, state::{CurrentState, StateFrame, SerializedStateVec, Direction, Facing, StateMap}, Fighter, systems::InputBuffer},
    Player, GGRSConfig, input::{BUFFER_SIZE, Action}, util::Buffer,
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

#[derive(Debug)]
pub struct PlayerHandles {
    pub state_list: Handle<SerializedStateVec>,
    pub fighter_data: Handle<FighterData>,
}

impl PlayerHandles {
    pub fn new(state_list: Handle<SerializedStateVec>, fighter_data: Handle<FighterData>) -> PlayerHandles {
        PlayerHandles { state_list, fighter_data }
    }
}

#[derive(Debug)]
pub struct PlayerHandleAccess(pub PlayerHandles, pub PlayerHandles);

impl PlayerHandleAccess {
    pub fn new(p1: PlayerHandles, p2: PlayerHandles) -> Self {
        Self(p1, p2)
    }
}

pub fn load_fighters(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    let state_list: Handle<SerializedStateVec> = asset_server.load("data/fighters/tahu/states.sl.json");
    let fighter_data: Handle<FighterData> = asset_server.load("data/fighters/tahu/fighter_data.json");

    let f2: Handle<FighterData> = asset_server.load("data/fighters/abe/fighter_data.json");


    loading.add(&state_list);
    loading.add(&fighter_data);
    loading.add(&f2);

    let p1 = PlayerHandles::new(state_list.clone(), fighter_data);
    let p2 = PlayerHandles::new(state_list, f2);
    let access = PlayerHandleAccess::new(p1, p2);

    commands.insert_resource(access);
    


}

pub fn spawn_fighters(
    mut commands: Commands, 
    //mut rip: ResMut<RollbackIdProvider>,
    sync_test_session: Option<Res<SyncTestSession<GGRSConfig>>>,

    handle_access: Res<PlayerHandleAccess>,
    //mut state_vecs: ResMut<Assets<SerializedStateVec>>,
    mut data: ResMut<Assets<FighterData>>,

    //asset_server: Res<AssetServer>

) {
    let num_players = sync_test_session
        .map(|s| s.num_players())
        .expect("Couldn't find Session");


    // println!("FighterData Assets: {:?}", data);
    // println!("Access: {:?}", handle_access);

    // println!("{:?}", asset_server.get_load_state(handle_access.0.fighter_data.id));

    let fighter1 = data.remove(&handle_access.0.fighter_data).expect("FighterData asset does not exist");
    let fighter2 = data.remove(&handle_access.1.fighter_data).expect("FighterData asset does not exist");

    let player1 = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            visibility: Visibility::visible(),
            transform: Transform::from_translation(Vec3::new(-2., 0., 0.)),
            ..default()
        })
        .insert(Name::new("Player 1"))
        .insert(Fighter)
        .insert(fighter1)
        .insert(CurrentState(0))
        .insert(Player(1))
        .insert(Facing(Direction::Right))
        .insert(StateFrame(0))
        .insert(InputBuffer(Buffer::with_capacity(BUFFER_SIZE)))

        .insert_bundle(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::U, Action::Lp),
                (KeyCode::I, Action::Mp),
                (KeyCode::O, Action::Hp),
                (KeyCode::J, Action::Lk),
                (KeyCode::K, Action::Mk),
                (KeyCode::L, Action::Hk),

                (KeyCode::A, Action::Left),
                (KeyCode::D, Action::Right),
                (KeyCode::W, Action::Up),
                (KeyCode::S, Action::Down)
            ])

        })
        .id();

    

    let player2 = 
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(2., 0., 0.)),
            ..default()
        })
        .insert(Name::new("Player 2"))
        .insert(Fighter)
        .insert(fighter2)
        .insert(CurrentState(0))
        .insert(Player(2))
        .insert(Facing(Direction::Left))
        //.insert(Rollback::new(rip.next_id()))
        .insert(StateFrame(0))

        .id();

    commands.insert_resource(PlayerEntities(player1, player2));

    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1. / 50.,
            ..default()
        },
        ..default()
    });


}
