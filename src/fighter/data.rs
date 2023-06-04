use bevy::gltf::Gltf;
use bevy::prelude::{Entity, Handle, ResMut, Resource, Vec3, Plugin, IntoSystemConfig, in_state};

use bevy::ecs::reflect::ReflectComponent;
use bevy::reflect::{Struct, TypeUuid};
use bevy::utils::HashMap;
use bevy::{prelude::Component, reflect::Reflect};

use bevy_asset_loader::prelude::*;
use parry3d::shape::{Capsule, Segment};
use serde::{Deserialize, Serialize};

use crate::game::NewGameState;

use super::hit::components::HitboxData;
use super::state::{HurtboxData, SerializedStateVec};

#[derive(Clone, Default, Debug, Serialize, Deserialize, Component, Reflect, TypeUuid)]
#[reflect(Component)]
#[uuid = "d99f5e90-13a4-11ed-861d-0242ac120002"]
pub struct FighterData {
    pub name: String,
    #[serde(alias = "states", default)]
    pub state_files: Vec<String>,

    #[serde(alias = "walkSpeed")]
    pub walk_speed: f32,
    #[serde(alias = "walkForward", default)]
    pub walk_forward: f32,
    #[serde(alias = "walkBack", default)]
    pub walk_back: f32,

    #[serde(flatten)]
    pub extra: HashMap<String, u8>,
}

// impl DynamicAsset for FighterData {
//     fn load(&self, asset_server: &bevy::prelude::AssetServer) -> Vec<bevy::prelude::HandleUntyped> {
//         todo!()
//     }

//     fn build(&self, world: &mut bevy::prelude::World) -> Result<bevy_asset_loader::prelude::DynamicAssetType, anyhow::Error> {
//         todo!()
//     }
// }

#[derive(AssetCollection, Resource)]
pub struct DataAssets {
    #[asset(key = "p1_data")]
    p1_fighter_data: Handle<FighterData>,
    #[asset(key = "p2_data")]
    p2_fighter_data: Handle<FighterData>,
}

pub fn fighter_data_setup(mut dynamic_assets: ResMut<DynamicAssets>) {
    dynamic_assets.register_asset(
        "p1_data",
        Box::new(StandardDynamicAsset::File {
            path: "data/fighters/ryo/ryo.fighter".to_owned(),
        }),
    );

    dynamic_assets.register_asset(
        "p2_data",
        Box::new(StandardDynamicAsset::File {
            path: "data/fighters/ryo/ryo.fighter".to_owned(),
        }),
    );
}

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_state::<NewGameState>()
            .add_loading_state(
                LoadingState::new(NewGameState::FighterDataLoading).continue_to_state(NewGameState::Next)
            )
            .add_collection_to_loading_state::<_, DataAssets>(NewGameState::FighterDataLoading)

            .add_system(fighter_data_setup.run_if(in_state(NewGameState::SetupData)))
            ;
    }
}


#[derive(AssetCollection, Resource)]
pub struct FighterAssets {
    #[asset(key = "p1_state_files", collection(typed))]
    p1_state_files: Vec<Handle<SerializedStateVec>>,
    #[asset(key = "p1_model")]
    p1_model: Handle<Gltf>,
}

#[derive(Component, Clone, Reflect)]
pub struct Collider {
    pub radius: f32,
    pub segment: SegmentProxy,
}

impl Default for Collider {
    fn default() -> Self {
        let shape = Capsule::new_y(1., 1.);
        shape.into()
    }
}

impl From<Capsule> for Collider {
    fn from(value: Capsule) -> Self {
        let radius = value.radius;
        let segment = value.segment.into();

        Collider { segment, radius }
    }
}

impl From<&Collider> for Capsule {
    fn from(value: &Collider) -> Self {
        let radius = value.radius;
        let segment = value.segment.clone().into();

        Capsule { segment, radius }
    }
}

#[derive(Clone, Reflect)]
pub struct SegmentProxy {
    pub a: Vec3,
    pub b: Vec3,
}

impl SegmentProxy {
    pub fn from_segment(segment: Segment) -> SegmentProxy {
        segment.into()
    }
}

impl From<SegmentProxy> for Segment {
    fn from(value: SegmentProxy) -> Self {
        Self {
            a: value.a.into(),
            b: value.b.into(),
        }
    }
}

impl From<Segment> for SegmentProxy {
    fn from(value: Segment) -> Self {
        Self {
            a: value.a.into(),
            b: value.b.into(),
        }
    }
}

pub struct CollisionData {
    pub attacker_box: HitboxData,
    pub attacker: Entity,
    pub recipient_box: HurtboxData,
    pub recipient: Entity,
}

impl CollisionData {
    pub fn get_attacker_priority(&self) -> u8 {
        self.attacker_box.priority
    }

    pub fn get_recipient(&self) -> Entity {
        self.recipient
    }
}

pub struct HitEvent(pub CollisionData);
