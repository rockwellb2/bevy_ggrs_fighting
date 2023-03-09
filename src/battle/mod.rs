use bevy::{
    core::Name,
    gltf::{Gltf},
    prelude::{
        default, shape, AssetServer, Assets, BuildChildren, Camera3dBundle, Color, Commands, Component, Entity,
        Handle, KeyCode, Mesh, NodeBundle, Parent, PbrBundle, PointLight,
        PointLightBundle, Query, Res, ResMut, Resource, StandardMaterial,
        TextBundle, Transform, Vec3, Visibility, With,
    },
    scene::{SceneBundle},
    text::{TextSection, TextStyle},
    ui::{
        AlignSelf, Display, FlexDirection, JustifyContent, PositionType, Size, Style, UiRect, Val,
    },
    utils::hashbrown::HashMap,
};

use bevy_ggrs::{Rollback, RollbackIdProvider, Session};


use ggrs::{SessionBuilder};


use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
    InputManagerBundle,
};
use parry3d::shape::{Capsule, Cuboid};

use crate::{
    fighter::{
        data::{Collider, FighterData},
        modifiers::{CreateObject, InputMet, InputWindowCheck, Object, OnExitSetPos},
        state::{
            ActiveHitboxes, BoneMap, CurrentState, Direction, Facing, Health, HurtboxData,
            Hurtboxes, Owner, PlayerAxis, ProjectileReference, SerializedStateVec,
            State as FightState, StateFrame, Velocity,
        },
        systems::InputBuffer,
        Fighter,
    },
    game::{GameState, RoundState},
    input::{Action, BUFFER_SIZE},
    util::Buffer,
    GGRSConfig, GameDebug, Player,
};

#[derive(Resource)]
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
    pub model: Handle<Gltf>,
}

impl PlayerHandles {
    pub fn new(
        state_list: Handle<SerializedStateVec>,
        fighter_data: Handle<FighterData>,
        model: Handle<Gltf>,
    ) -> PlayerHandles {
        PlayerHandles {
            state_list,
            fighter_data,
            model,
        }
    }
}

#[derive(Debug, Resource)]
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
    let state_list: Handle<SerializedStateVec> =
        //asset_server.load("data/fighters/tahu/states.sl.json");
        asset_server.load("data/fighters/ryo/ryo.states");
    let fighter_data: Handle<FighterData> =
        asset_server.load("data/fighters/tahu/fighter_data.json");
    //let model: Handle<Gltf> = asset_server.load("models/sfv_ryu.glb");
    let model: Handle<Gltf> = asset_server.load("models/ryo_maybe_messed_up.glb");

    let f2: Handle<FighterData> = asset_server.load("data/fighters/abe/fighter_data.json");

    let p1 = PlayerHandles::new(state_list.clone(), fighter_data, model.clone());
    let p2 = PlayerHandles::new(state_list, f2, model);
    let access = PlayerHandleAccess::new(p1, p2);

    commands.insert_resource(access);
}

pub fn loading_wait(
    asset_server: Res<AssetServer>,
    mut state: ResMut<RoundState>,

    player_access: Res<PlayerHandleAccess>,
) {
    let handles = vec![
        player_access.0.fighter_data.id(),
        player_access.0.state_list.id(),
        player_access.1.fighter_data.id(),
        player_access.1.state_list.id(),
        player_access.0.model.id(),
        player_access.1.model.id(),
    ];

    println!("LOADING...");

    match asset_server.get_group_load_state(handles) {
        bevy::asset::LoadState::Loaded => *state = RoundState::ExitLoading,
        _ => return,
    }
}

#[derive(Resource)]
pub struct HitboxMaterial(pub Handle<StandardMaterial>);

#[derive(Resource)]
pub struct HurtboxMaterial(pub Handle<StandardMaterial>);

#[allow(clippy::too_many_arguments)]
pub fn spawn_fighters(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    handle_access: Res<PlayerHandleAccess>,
    mut data: ResMut<Assets<FighterData>>,
    assets_gltf: Res<Assets<Gltf>>,

    mut state: ResMut<RoundState>,
    debug: Res<GameDebug>,
) {
    let fighter1 = data
        .remove(&handle_access.0.fighter_data)
        .expect("FighterData asset does not exist");
    let fighter2 = data
        .remove(&handle_access.1.fighter_data)
        .expect("FighterData asset does not exist");

    let mut hurt_mat: StandardMaterial = Color::rgba(1., 1., 0., 0.3).into();
    hurt_mat.unlit = true;
    hurt_mat.cull_mode = None;
    let hurtbox_material = materials.add(hurt_mat);

    let mut hit_mat: StandardMaterial = Color::rgba(1., 0., 0., 0.3).into();
    hit_mat.unlit = true;
    hit_mat.cull_mode = None;
    let hitbox_material = materials.add(hit_mat);

    commands.insert_resource(HitboxMaterial(hitbox_material));
    commands.insert_resource(HurtboxMaterial(hurtbox_material));

    let player1 = commands
        .spawn(SceneBundle {
            scene: assets_gltf
                .get(&handle_access.0.model)
                .expect("Asset doesn't exist")
                .scenes[0]
                .clone(),
            transform: Transform {
                translation: (-2., 0., 0.).into(),
                //scale: Vec3::splat(3.),
                ..default()
            }
            .looking_at((2., 0., 0.).into(), Vec3::Y),
            ..default()
        })
        .insert(Name::new("Player 1"))
        .insert(Fighter)
        .insert(fighter1)
        //.insert(Rollback::new(rip.next_id()))
        .insert(CurrentState(0))
        .insert(Player(1))
        .insert(Facing(Direction::Right))
        .insert(StateFrame(0))
        .insert(InputBuffer(Buffer::with_capacity(BUFFER_SIZE)))
        .insert(BoneMap(HashMap::new()))
        .insert(ActiveHitboxes(Vec::new()))
        .insert(Health(500))
        .insert(Velocity(Vec3::ZERO))
        .insert(Hurtboxes::new())
        .insert(PlayerAxis {
            opponent_pos: Vec3::new(2., 0., 0.),
            x: Vec3::X,
            z: Vec3::Z,
        })
        .insert(InputManagerBundle::<Action> {
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
                (KeyCode::S, Action::Down),
            ]),
        })
        .id();

    let player2 = commands
        .spawn(SceneBundle {
            scene: assets_gltf
                .get(&handle_access.1.model)
                .expect("Asset doesn't exist")
                .scenes[0]
                .clone(),
            transform: Transform {
                translation: (2., 0., 0.).into(),
                //scale: Vec3::splat(3.),
                ..default()
            }
            .looking_at((-2., 0., 0.).into(), Vec3::Y),
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
        .insert(BoneMap(HashMap::new()))
        .insert(Health(500))
        .insert(Velocity(Vec3::ZERO))
        .insert(Hurtboxes::new())
        .insert(PlayerAxis {
            opponent_pos: Vec3::new(-2., 0., 0.),
            x: Vec3::X,
            z: Vec3::Z,
        })
        .id();

    if !debug.0 {
        commands
            .entity(player1)
            .insert(Rollback::new(rip.next_id()));
        commands
            .entity(player2)
            .insert(Rollback::new(rip.next_id()));
    }

    commands.insert_resource(PlayerEntities(player1, player2));

    commands.spawn(PointLightBundle {
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

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.8, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(MatchCamera);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 100. })),
            material: materials.add(Color::WHITE.into()),
            ..default()
        })
        .insert(Name::new("Ground"));

    *state = RoundState::Armature
    //*state = RoundState::SomethingElseEntirely;
}

#[derive(Component)]
pub struct MatchCamera;

#[allow(clippy::too_many_arguments)]
pub fn extra_setup_system(
    object_query: Query<(&CreateObject, &Owner)>,
    mut parent_query: Query<(&Transform, Option<&mut ProjectileReference>)>,

    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,

    mut round_state: ResMut<RoundState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    exit_set_pos_query: Query<(&OnExitSetPos, &Owner)>,

    mut state_query: Query<(&mut FightState, &Owner)>,
    mut bonemap_query: Query<&mut BoneMap>,
    bone_name_query: Query<(&Name, Entity), With<Transform>>,

    window_check_query: Query<Entity, (With<FightState>, With<InputWindowCheck>)>,

    bone_parent_query: Query<&Parent>,

    player_query: Query<Entity, (With<Player>, With<Fighter>)>,
) {
    let projectile_material = materials.add(Color::rgba(0., 1., 0., 0.5).into());
    println!("Start of projectile system");

    for (create_object, owner) in object_query.iter() {
        println!("Does this happen in the projectile system?");
        match &create_object.0 {
            Object::Projectile(projectile) => {
                // let shape = shapes::Rectangle {
                //     extents: projectile.dimensions.truncate(),
                //     origin: RectangleOrigin::Center,
                // };

                //let cuboid = Cuboid::new((projectile.dimensions / 2.).into());
                let capsule =
                    Capsule::new_y(projectile.dimensions.y / 2., projectile.dimensions.x / 2.);

                let mut ids = Vec::new();

                for _ in 0..projectile.max {
                    let collider: Collider = capsule.into();

                    let entity = commands
                        // .spawn_bundle(GeometryBuilder::build_as(
                        //     &shape,
                        //     DrawMode::Fill(FillMode::color(Color::rgba(1., 0., 0., 0.8))),
                        //     Transform::default()
                        // ))
                        .spawn(PbrBundle {
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
                        .insert(collider)
                        .insert(Velocity(projectile.start_velocity))
                        .insert(Rollback::new(rip.next_id()))
                        .insert(StateFrame(0))
                        .insert(Owner(owner.get()))
                        .id();

                    ids.push((entity, false))
                }

                if let Ok((_tf, projectile_ref)) = parent_query.get_mut(owner.get()) {
                    if let Some(mut projectile_ref) = projectile_ref {
                        projectile_ref.insert_ids(projectile.name.clone(), ids);
                        projectile_ref
                            .amount_in_use
                            .insert(projectile.name.clone(), 0);
                    } else {
                        let mut projectile_ref = ProjectileReference::new();
                        projectile_ref.insert_ids(projectile.name.clone(), ids);
                        projectile_ref
                            .amount_in_use
                            .insert(projectile.name.clone(), 0);

                        commands.entity(owner.get()).insert(projectile_ref);
                    }
                }
            }
            Object::None => panic!(),
        }
    }

    for (set_pos, owner) in exit_set_pos_query.iter() {
        let bone_name = &set_pos.bone;

        if let Ok(mut bonemap) = bonemap_query.get_mut(owner.get()) {
            if let Some(_bone_entity) = bonemap.0.get(bone_name) {
                //hitbox.bone_entity = Some(*bone_entity);
            } else {
                for (name, bone_entity) in bone_name_query.iter() {
                    if &name.to_string() == bone_name {
                        let mut ancestor = bone_parent_query
                            .get(bone_entity)
                            .expect("Bone doesn't have parent");
                        loop {
                            if let Ok(bone_parent) = bone_parent_query.get(ancestor.get()) {
                                ancestor = bone_parent;
                            } else {
                                break;
                            }
                        }

                        if owner.get() == ancestor.get() {
                            bonemap.0.insert(bone_name.to_string(), bone_entity);
                            //hitbox.bone_entity = Some(bone_entity);
                            println!("It somehow got here, doing bone things");
                            break;
                        }
                    }
                }
            }
        }
    }

    for (mut fight_state, parent) in state_query.iter_mut() {
        if let Some(hitboxes) = &mut fight_state.hitboxes {
            for boxes in hitboxes.values_mut() {
                for hitbox in boxes {
                    let bone_name = hitbox.bone.clone();

                    if let Ok(mut bonemap) = bonemap_query.get_mut(parent.get()) {
                        if let Some(bone_entity) = bonemap.0.get(&bone_name) {
                            hitbox.bone_entity = Some(*bone_entity);
                        } else {
                            for (name, bone_entity) in bone_name_query.iter() {
                                if name.to_string() == bone_name {
                                    let mut ancestor = bone_parent_query
                                        .get(bone_entity)
                                        .expect("Bone doesn't have parent");
                                    loop {
                                        if let Ok(bone_parent) =
                                            bone_parent_query.get(ancestor.get())
                                        {
                                            ancestor = bone_parent;
                                        } else {
                                            break;
                                        }
                                    }

                                    if parent.get() == ancestor.get() {
                                        bonemap.0.insert(bone_name, bone_entity);
                                        hitbox.bone_entity = Some(bone_entity);
                                        println!("It somehow got here, doing bone things");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for check_input_state in window_check_query.iter() {
        commands.entity(check_input_state).insert(InputMet(false));
    }

    // for player_ent in player_query.iter() {
    //     commands
    //         .entity(player_ent)
    //         .insert(Rollback::new(rip.next_id()));
    // }

    let sess_build = SessionBuilder::<GGRSConfig>::new()
        //.with_max_prediction_window(8)
        .with_check_distance(2)
        .with_input_delay(2)
        .with_num_players(2)
        .add_player(ggrs::PlayerType::Local, 0)
        .unwrap()
        .add_player(ggrs::PlayerType::Local, 1)
        .unwrap();

    let sess = sess_build
        .start_synctest_session()
        .expect("Couldn't start Session");
    commands.insert_resource(Session::SyncTestSession(sess));

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
            current: full,
        }
    }

    pub fn health_percent(&self) -> f32 {
        (self.current as f32 / self.full as f32) * 100.
    }
}

#[derive(Component)]
pub struct ChangeText;

pub fn create_battle_ui(
    mut commands: Commands,
    mut state: ResMut<RoundState>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(Name::new("UI Parent"))
        .with_children(|parent| {
            // Player 1
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: bevy::ui::FlexDirection::Column,
                        size: Size::new(Val::Percent(45.), Val::Percent(20.)),
                        align_self: AlignSelf::FlexStart,
                        display: Display::Flex,
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .insert(Name::new("Player 1 UI"))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
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
                            background_color: Color::BLACK.into(),
                            ..default()
                        })
                        .insert(Name::new("Player 1 Lifebar"))
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                                        align_self: AlignSelf::FlexEnd,
                                        ..default()
                                    },
                                    background_color: Color::GREEN.into(),
                                    ..default()
                                })
                                .insert(Player(1))
                                .insert(Lifebar::new(500))
                                .insert(Name::new("Player 1 Lifebar Fill"));
                        });
                });

            // Player 2
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: bevy::ui::FlexDirection::Column,
                        size: Size::new(Val::Percent(45.), Val::Percent(20.)),
                        align_self: AlignSelf::FlexStart,
                        display: Display::Flex,
                        ..default()
                    },
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .insert(Name::new("Player 2 UI"))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
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
                            background_color: Color::BLACK.into(),
                            ..default()
                        })
                        .insert(Name::new("Player 2 Lifebar"))
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                                        align_self: AlignSelf::FlexEnd,
                                        ..default()
                                    },
                                    background_color: Color::GREEN.into(),
                                    ..default()
                                })
                                .insert(Player(2))
                                .insert(Lifebar::new(500))
                                .insert(Name::new("Player 2 Lifebar Fill"));
                        });
                });
        });

    commands
        .spawn(
            TextBundle::from_sections([
                TextSection::new(
                    "State: ",
                    TextStyle {
                        font: font.clone(),
                        font_size: 30.0,
                        color: Color::YELLOW,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font,
                    font_size: 30.0,
                    color: Color::YELLOW,
                }),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ChangeText);

    *state = RoundState::Loading
}
