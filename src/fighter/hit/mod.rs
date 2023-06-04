pub mod components {
    use bevy::{
        prelude::{Component, Entity, Handle, Mesh, ReflectComponent, Vec3},
        reflect::{FromReflect, Reflect},
    };
    use serde::{
        Deserialize, Serialize,
    };

    use super::super::state::{Frame, FrameWindow, HBox, HitLevel};

    #[derive(Default, Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component)]
    #[reflect(Component)]
    pub struct HitboxData {
        #[serde(default)]
        pub priority: u8,
        #[serde(default)]
        pub id: Option<usize>,
        #[serde(default)]
        pub global_id: Option<u32>,
        pub bone: String,
        #[serde(default)]
        pub bone_entity: Option<Entity>,
        pub radius: f32,
        #[serde(alias = "halfHeight")]
        pub half_height: f32,
        #[serde(default)]
        pub offset: Vec3,
        #[serde(default, deserialize_with = "super::super::state::deserialize_rotation")]
        pub rotation: (f32, f32),
        pub damage: u16,
        #[serde(alias = "onHit", default)]
        pub on_hit: OnHit,
        #[serde(alias = "onAirHit", default = "HitboxData::on_air_hit_default")]
        pub on_air_hit: OnHit,
        pub blockstun: Frame,
        window: FrameWindow,
        #[serde(default)]
        rehit: Option<u16>, // Number frames after hitting that hitbox can hit again,
        #[serde(alias = "hitLevel", default)]
        hit_level: HitLevel,
    }

    impl HitboxData {
        pub fn get_end_frame(&self) -> Frame {
            self.window
                .end
                .expect("End frame does not exist in HitboxData")
        }

        pub fn get_start_frame(&self) -> Frame {
            self.window
                .start
                .expect("End frame does not exist in HitboxData")
        }
    }

    impl HitboxData {
        fn on_air_hit_default() -> OnHit {
            OnHit::Launch(Vec3::default())
        }

        pub fn mesh_default() -> Option<Handle<Mesh>> {
            None
        }

        pub fn set_global_id(&mut self, global_id: u32) {
            self.global_id = Some(global_id);
        }
    }

    impl HBox for HitboxData {
        fn get_priority(&self) -> u8 {
            self.priority
        }

        fn get_offset(&self) -> Vec3 {
            self.offset
        }

        fn set_id(&mut self, value: usize) {
            self.id = Some(value);
        }
    }


    #[derive(Debug, Serialize, Deserialize, Clone, FromReflect, Reflect, Component)]
    pub enum OnHit {
        Launch(Vec3),
        Grounded { kb: Vec3, hitstun: Frame },
        Stun(Frame)
    }

    impl Default for OnHit {
        fn default() -> Self {
            Self::Grounded { kb: Vec3::default(), hitstun: 1 }
        }
    }

    #[derive(Serialize, Deserialize, Default, Debug, Component, Reflect, Clone)]
    #[reflect(Component)]
    pub struct AirborneHitstun;
}



