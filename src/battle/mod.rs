use bevy::{
    core::Name,
    math::Vec2,
    prelude::{default, Color, Commands, Entity, ResMut, Res, AssetServer, Handle, Assets, Camera2dBundle, OrthographicProjection, Visibility, Transform, Vec3, KeyCode, NodeBundle, BuildChildren, Component, State, Query, Parent, SpatialBundle, VisibilityBundle, ComputedVisibility, PbrBundle, Mesh, shape, StandardMaterial, Camera3dBundle, PointLightBundle, PointLight, Children, DespawnRecursiveExt},
    sprite::{Sprite, SpriteBundle}, ui::{Style, Size, Val, Display, JustifyContent, AlignSelf, UiRect, FlexDirection}, scene::{SceneBundle, Scene}, gltf::{Gltf, GltfExtras}, ecs::{world::EntityRef, system::EntityCommands}
};

use bevy_ggrs::{Rollback, RollbackIdProvider};
use bevy_prototype_lyon::{prelude::{tess::geom::euclid::num::Round, GeometryBuilder, DrawMode, FillMode}, shapes::{RectangleOrigin, self}};
use bevy_scene_hook::{HookedSceneBundle, SceneHook};
use ggrs::{SyncTestSession, P2PSession};

use iyes_progress::prelude::AssetsLoading;
use leafwing_input_manager::{InputManagerBundle, prelude::{ActionState, InputMap}};
use parry3d::shape::{Cuboid, Capsule};


use crate::{
    fighter::{data::{FighterData, Collider}, state::{CurrentState, StateFrame, SerializedStateVec, Direction, Facing, Health, Owner, ProjectileReference, Velocity, PlayerAxis, HurtboxData, Hurtboxes}, Fighter, systems::InputBuffer, modifiers::{CreateObject, Object}},
    Player, GGRSConfig, input::{BUFFER_SIZE, Action}, util::Buffer, game::{GameState, RoundState}, GameDebug,
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

impl From<&PlayerEntities> for [Entity; 2] {
    fn from(value: &PlayerEntities) -> Self {
        [value.0, value.1]
    }
}

#[derive(Debug)]
pub struct PlayerHandles {
    pub state_list: Handle<SerializedStateVec>,
    pub fighter_data: Handle<FighterData>,
    pub model: Handle<Gltf>
}

impl PlayerHandles {
    pub fn new(
        state_list: Handle<SerializedStateVec>, 
        fighter_data: Handle<FighterData>,
        model: Handle<Gltf>
    ) -> PlayerHandles 
    {
        PlayerHandles { state_list, fighter_data, model }
    }
}

#[derive(Debug)]
pub struct PlayerHandleAccess(pub PlayerHandles, pub PlayerHandles);

impl PlayerHandleAccess {
    pub fn new(p1: PlayerHandles, p2: PlayerHandles) -> Self {
        Self(p1, p2)
    }

    pub fn get(&self, id: u8) -> &PlayerHandles {
        match id {
            1 => &self.0,
            2 => &self.1,
            _ => panic!(),
        }
    }
}

pub fn load_fighters(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    // mut loading: ResMut<AssetsLoading>,
) {
    let state_list: Handle<SerializedStateVec> = asset_server.load("data/fighters/tahu/states.sl.json");
    let fighter_data: Handle<FighterData> = asset_server.load("data/fighters/tahu/fighter_data.json");
    let model: Handle<Gltf> = asset_server.load("models/ryu_new.glb");


    let f2: Handle<FighterData> = asset_server.load("data/fighters/abe/fighter_data.json");


    let p1 = PlayerHandles::new(state_list.clone(), fighter_data, model.clone());
    let p2 = PlayerHandles::new(state_list, f2, model);
    let access = PlayerHandleAccess::new(p1, p2);

    commands.insert_resource(access);


}

pub fn loading_wait(
    asset_server: Res<AssetServer>,
    mut state: ResMut<RoundState>,


    player_access: Res<PlayerHandleAccess>
) {
    let handles = vec![
        player_access.0.fighter_data.id,
        player_access.0.state_list.id,
        player_access.1.fighter_data.id,
        player_access.1.state_list.id,
        player_access.0.model.id,
        player_access.1.model.id,
    ];

    println!("LOADING...");

    match asset_server.get_group_load_state(handles) {
        bevy::asset::LoadState::Loaded => {
            *state = RoundState::ExitLoading
        },
        _ => return
    }
}


pub fn spawn_fighters(
    mut commands: Commands, 
    mut rip: ResMut<RollbackIdProvider>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    handle_access: Res<PlayerHandleAccess>,
    mut data: ResMut<Assets<FighterData>>,
    assets_gltf: Res<Assets<Gltf>>,

    mut state: ResMut<RoundState>,
    debug: Res<GameDebug>
) {
    let fighter1 = data.remove(&handle_access.0.fighter_data).expect("FighterData asset does not exist");
    let fighter2 = data.remove(&handle_access.1.fighter_data).expect("FighterData asset does not exist");

    let mut hurt_mat: StandardMaterial = Color::rgba(1., 1., 0., 0.3).into();
    hurt_mat.unlit = true;
    hurt_mat.cull_mode = None;

    //let hitbox_material = materials.add(Color::rgba(1., 0., 0., 0.3).into());
    let hurtbox_material = materials.add(hurt_mat);

    let hook = move |entity: &EntityRef, cmds: &mut EntityCommands| {
        if let Some(extras) = entity.get::<GltfExtras>() {

            let hurt: HurtboxData = serde_json::de::from_str(extras.value.as_str()).expect("Could not deserialize as HurtboxData");
            let capsule = Capsule::new_y(hurt.half_height - hurt.radius, hurt.radius);
            cmds
                .insert(hurt)
                .insert(Collider { shape: capsule });

            let commands = cmds.commands();

            let children  = entity.get::<Children>().expect("Entity does not have Children");
            for child in children.iter() {
                commands.entity(*child)
                    .insert(hurtbox_material.clone());
            }
        }
    };



    let player1 = commands
        .spawn_bundle(HookedSceneBundle {
            scene: SceneBundle {
                scene: assets_gltf.get(&handle_access.0.model).expect("Asset doesn't exist").scenes[0].clone(),
                transform: Transform { 
                    translation: (-2., 0., 0.).into(),  
                    //scale: Vec3::splat(3.),
                    ..default()
                }.looking_at((2., 0., 0.).into(), Vec3::Y),
                ..default()
            },
            hook: SceneHook::new(hook.clone())
        })
        .insert(Name::new("Player 1"))
        .insert(Fighter)
        .insert(fighter1)
        .insert(Rollback::new(rip.next_id()))
        .insert(CurrentState(0))
        .insert(Player(1))
        .insert(Facing(Direction::Right))
        .insert(StateFrame(0))
        .insert(InputBuffer(Buffer::with_capacity(BUFFER_SIZE)))
        .insert(Health(500))
        .insert(Velocity(Vec3::ZERO))
        .insert(Hurtboxes::new())
        .insert(PlayerAxis {
            opponent_pos: Vec3::new(2., 0., 0.),
            x: Vec3::X,
            z: Vec3::Z
        })

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

        .spawn_bundle(HookedSceneBundle {
            scene: SceneBundle {
                scene: assets_gltf.get(&handle_access.1.model).expect("Asset doesn't exist").scenes[0].clone(),
                transform: Transform { 
                    translation: (2., 0., 0.).into(),  
                    //scale: Vec3::splat(3.),
                    ..default()
                }.looking_at((-2., 0., 0.).into(), Vec3::Y),
                ..default()
            },
            hook: SceneHook::new(hook)
        })
        .insert(Name::new("Player 2"))
        .insert(Fighter)
        .insert(fighter2)
        .insert(Rollback::new(rip.next_id()))
        .insert(CurrentState(0))
        .insert(Player(2))
        .insert(Facing(Direction::Left))
        .insert(StateFrame(0))
        .insert(InputBuffer(Buffer::with_capacity(BUFFER_SIZE)))
        .insert(Health(500))
        .insert(Velocity(Vec3::ZERO))
        .insert(Hurtboxes::new())
        .insert(PlayerAxis {
            opponent_pos: Vec3::new(-2., 0., 0.),
            x: Vec3::X,
            z: Vec3::Z
        })
        .id();

    if !debug.0 {
        commands.entity(player1).insert(Rollback::new(rip.next_id()));
        commands.entity(player2).insert(Rollback::new(rip.next_id()));
    }

    commands.insert_resource(PlayerEntities(player1, player2));

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 30_000.,
            shadows_enabled: true,
            radius: 50.,
            range: 100.,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 20.0, 4.0),
        ..default()
    });

    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.8, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
    .insert(MatchCamera);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100. })),
        material: materials.add(Color::WHITE.into()),
        ..default()
    })
    .insert(Name::new("Ground"));

    *state = RoundState::Armature


}

#[derive(Component)]
pub struct MatchCamera;


pub fn extra_setup_system(
    object_query: Query<(&CreateObject, &Parent)>,
    mut parent_query: Query<(&Transform, Option<&mut ProjectileReference>)>,

    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,

    mut round_state: ResMut<RoundState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>

) {
    let projectile_material = materials.add(Color::rgba(0., 1., 0., 0.5).into());


    for (create_object, parent) in object_query.iter() {
        match &create_object.0 {
            Object::Projectile(projectile) => {
                // let shape = shapes::Rectangle {
                //     extents: projectile.dimensions.truncate(),
                //     origin: RectangleOrigin::Center,
                // };

                //let cuboid = Cuboid::new((projectile.dimensions / 2.).into());
                let capsule = Capsule::new_y(projectile.dimensions.y / 2., projectile.dimensions.x / 2.);


                let mut ids = Vec::new();

                for _ in 0..projectile.max {
                    let entity = commands
                        // .spawn_bundle(GeometryBuilder::build_as(
                        //     &shape,
                        //     DrawMode::Fill(FillMode::color(Color::rgba(1., 0., 0., 0.8))),
                        //     Transform::default()
                        // ))
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Capsule {
                                radius: projectile.dimensions.x,
                                depth: 0.,
                                ..default()
                            })),
                            material: projectile_material.clone(),
                            visibility: Visibility { is_visible: false },
                            ..default()
                        })
                        // .insert_bundle(VisibilityBundle {
                        //     visibility: Visibility { is_visible: false },
                        //     computed: ComputedVisibility::default()
                        //})
                        .insert(Name::new(projectile.name.clone()))
                        .insert(projectile.clone())
                        .insert(Collider { shape: capsule })
                        .insert(Velocity(projectile.start_velocity))
                        .insert(Rollback::new(rip.next_id()))
                        .insert(StateFrame(0))
                        .insert(Owner(parent.get()))
                        .id()
                        ;

                    ids.push((entity, false))
                }

                if let Ok((_tf, projectile_ref)) = parent_query.get_mut(parent.get()) {
                    if let Some(mut projectile_ref) = projectile_ref {
                        projectile_ref.insert_ids(projectile.name.clone(), ids);
                        projectile_ref.amount_in_use.insert(projectile.name.clone(), 0);
                    }
                    else {
                        let mut projectile_ref = ProjectileReference::new();
                        projectile_ref.insert_ids(projectile.name.clone(), ids);
                        projectile_ref.amount_in_use.insert(projectile.name.clone(), 0);

                        commands.entity(parent.get())
                            .insert(projectile_ref);
                    }

                    
                }
            },
            Object::None => panic!(),
        }
    }

    *round_state = RoundState::Round;

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
    mut state: ResMut<RoundState>

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

    *state = RoundState::Loading


}