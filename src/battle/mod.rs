use std::collections::VecDeque;

use bevy::{
    core::Name,
    math::Vec2,
    prelude::{default, Color, Commands, Entity, ResMut, Res, AssetServer, Handle, Assets, Camera2dBundle, OrthographicProjection, Visibility, Transform, Vec3, KeyCode, NodeBundle, BuildChildren, Component},
    sprite::{Sprite, SpriteBundle}, ui::{Style, UiImage, Size, Val, Display, JustifyContent, AlignSelf, UiRect, FlexDirection}
};

use bevy_ggrs::{Rollback, RollbackIdProvider};
use ggrs::{SyncTestSession, P2PSession};

use iyes_progress::prelude::AssetsLoading;
use leafwing_input_manager::{InputManagerBundle, prelude::{ActionState, InputMap}};


use crate::{
    fighter::{data::FighterData, state::{CurrentState, StateFrame, SerializedStateVec, Direction, Facing, StateMap, Health}, Fighter, systems::InputBuffer},
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
    mut rip: ResMut<RollbackIdProvider>,
    sync_test_session: Option<Res<SyncTestSession<GGRSConfig>>>,
    p2p_session: Option<Res<P2PSession<GGRSConfig>>>,

    handle_access: Res<PlayerHandleAccess>,
    //mut state_vecs: ResMut<Assets<SerializedStateVec>>,
    mut data: ResMut<Assets<FighterData>>,

    //asset_server: Res<AssetServer>

) {
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
        .insert(Health(500))

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
        .insert(StateFrame(0))
        .insert(InputBuffer(Buffer::with_capacity(BUFFER_SIZE)))
        .insert(Health(500))
        

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

#[derive(Component)]
pub struct Lifebar {
    full: u16,
    pub current: u16,
}

impl Lifebar {
    pub fn new(full: u16) -> Self {
        Lifebar {
            full,
            current: full
        }
    }

    pub fn health_percent(&self) -> f32 {
        (self.current as f32 / self.full as f32) * 100.
    }
}

pub fn create_battle_ui(
    mut commands: Commands,

) {
    commands.spawn_bundle(
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(Name::new("UI Parent"))
        .with_children(|parent| {
            // Player 1
            parent.spawn_bundle(
                NodeBundle {
                    style: Style {
                        flex_direction: bevy::ui::FlexDirection::ColumnReverse,
                        size: Size::new(Val::Percent(45.), Val::Percent(20.)),
                        align_self: AlignSelf::FlexEnd,
                        display: Display::Flex,
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                }
            )
            .insert(Name::new("Player 1 UI"))
            .with_children(|parent| {
                parent.spawn_bundle(
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::RowReverse,
                            size: Size::new(Val::Percent(85.), Val::Percent(20.)),
                            position: UiRect {
                                top: Val::Percent(30.),
                                ..default()
                            },
                            align_self: AlignSelf::FlexEnd,
                            ..default()
                        },
                        color: Color::BLACK.into(),
                        ..default()
                    }
                )
                .insert(Name::new("Player 1 Lifebar"))
                .with_children(|parent| {
                    parent.spawn_bundle(
                        NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                                align_self: AlignSelf::FlexEnd,
                                ..default()
                            },
                            color: Color::GREEN.into(),
                            ..default()
                        }
                    )
                    .insert(Player(1))
                    .insert(Lifebar::new(500))
                    .insert(Name::new("Player 1 Lifebar Fill"));
                });
            });

            // Player 2
            parent.spawn_bundle(
                NodeBundle {
                    style: Style {
                        flex_direction: bevy::ui::FlexDirection::ColumnReverse,
                        size: Size::new(Val::Percent(45.), Val::Percent(20.)),
                        align_self: AlignSelf::FlexEnd,
                        display: Display::Flex,
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                }
            )
            .insert(Name::new("Player 2 UI"))
            .with_children(|parent| {
                parent.spawn_bundle(
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            size: Size::new(Val::Percent(85.), Val::Percent(20.)),
                            position: UiRect {
                                top: Val::Percent(30.),
                                ..default()
                            },
                            align_self: AlignSelf::FlexStart,
                            ..default()
                        },
                        color: Color::BLACK.into(),
                        ..default()
                    }
                )
                .insert(Name::new("Player 2 Lifebar"))
                .with_children(|parent| {
                    parent.spawn_bundle(
                        NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                                align_self: AlignSelf::FlexEnd,
                                ..default()
                            },
                            color: Color::GREEN.into(),
                            ..default()
                        }
                    )
                    .insert(Player(2))
                    .insert(Lifebar::new(500))
                    .insert(Name::new("Player 2 Lifebar Fill"));
                });
            });
        });

}