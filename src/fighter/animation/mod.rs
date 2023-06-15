pub(crate) mod setup {
    use std::default;
    use std::f32::consts::{FRAC_PI_2, PI};
    use std::mem::size_of;
    use std::sync::Arc;

    use super::components::{AnimEntity, BoneTransforms, FullBoneTransformMap, TransformListRef};
    use crate::battle::{HurtboxMaterial, PlayerEntities};
    use crate::fighter::data::Collider;
    use crate::fighter::state::{BoneMap, FighterPosition};

    use bevy::transform::TransformBundle;
    use bevy::utils::HashMap;
    use bevy::{
        gltf::GltfExtras,
        prelude::{AnimationPlayer, Children, HierarchyQueryExt},
    };
    use parry3d::shape::Capsule;

    use super::super::state::Owner;

    use super::super::state::Hurtboxes;
    use super::components::PositionEntity;

    use crate::game::RoundState;

    use bevy::prelude::{
        BuildChildren, Mat4, Quat, ResMut, SpatialBundle, Transform, Vec3, VisibilityBundle,
    };

    use bevy::prelude::Parent;

    use super::super::state::HurtboxData;

    use super::components::Animation;

    use bevy::prelude::AnimationClip;

    use super::super::Fighter;

    use bevy::prelude::With;

    use super::super::state::StateMap;

    use crate::Player;

    use bevy::prelude::Name;

    use bevy::prelude::Entity;

    use bevy::prelude::Query;

    use crate::battle::PlayerHandleAccess;

    use bevy::gltf::Gltf;

    use bevy::prelude::{default, Assets};

    use bevy::prelude::Res;

    use bevy::prelude::Commands;

    /// A setup function that iterates over all entities with a `StateMap`, then iterates over
    /// that entity's state entities, inserting an `Animation` component in each
    ///
    /// # Arguments
    /// * `commands` - Bevy commands struct,
    /// * `assets_gltf` - The GLTF resources loaded by Bevy,
    /// * `handle_access` - The resource that contains relevant handles for assets
    /// * `query` - A query to get the Entity and Name of a state entity
    /// * `fighter_query` - A query to get every Fighter entity
    /// * `animation_clips` - The AnimationClip resources loaded by Bevy through the GLTF importer
    ///
    ///
    pub fn insert_animations(
        mut commands: Commands,
        assets_gltf: Res<Assets<Gltf>>,
        handle_access: Res<PlayerHandleAccess>,
        query: Query<(Entity, &Name)>,
        fighter_query: Query<(&Player, &StateMap), With<Fighter>>,
        animation_clips: Res<Assets<AnimationClip>>,
    ) {
        for (player, map) in fighter_query.iter() {
            let handle = handle_access.get(player.0).model.clone();
            let gltf = assets_gltf.get(&handle).expect("GLTF handle doesn't exist");
            let animations = &gltf.named_animations;
            for (entity, name) in query.iter_many(map.map.values()) {
                if let Some(animation) = animations.get(
                    name.to_string()
                        .rsplit(") ")
                        .next()
                        .expect("Split didn't work"),
                ) {
                    let length = animation_clips
                        .get(animation)
                        .expect("AnimationClip doesn't exist")
                        .duration();

                    let get_rid_of_this = animation_clips
                        .get(animation)
                        .expect("AnimationClip doesn't exist")
                        .curves();

                    commands
                        .entity(entity)
                        .insert(Animation(animation.clone(), length));
                }
            }
        }
    }

    /// A setup function that looks at the extra GLTF properties and extracts data from them.
    /// Currently, it looks at nodes with "Hurt" in them to transform them into Hurtboxes,
    /// and nodes with the property "fighterPosition" to set it as the reference for the fighter's position.
    ///
    /// # Note
    ///
    /// Ordering of systems is a problem, where sometimes the `HurtboxMaterial` is not loaded when the resource is grabbed.
    /// Current solution is hacky, and only works because `armature_system` only moves to next set after `HurtboxData` is found.
    pub fn insert_hurtbox_data(
        mut commands: Commands,
        fighter_query: Query<Entity, With<Fighter>>,
        children_query: Query<&Children>,
        extras_query: Query<(&GltfExtras, &Name)>,

        hurtbox_material: Option<Res<HurtboxMaterial>>,
    ) {
        if hurtbox_material.is_none() {
            return;
        }

        let hurtbox_material = &hurtbox_material.unwrap().0;

        for player in fighter_query.iter() {
            for descendent in children_query.iter_descendants(player) {
                if let Ok((extras, name)) = extras_query.get(descendent) {
                    if name.contains("Hurt") {
                        let hurt: HurtboxData = serde_json::de::from_str(extras.value.as_str())
                            .expect("Could not deserialize as HurtboxData");

                        let capsule = Capsule::new_y(hurt.half_height - hurt.radius, hurt.radius);
                        let collider: Collider = capsule.into();
                        commands.entity(descendent).insert((hurt, collider));

                        if let Ok(children) = children_query.get(descendent) {
                            for child in children.iter() {
                                commands.entity(*child).insert(hurtbox_material.clone());
                            }
                        }
                    }

                    if extras.value.contains("fighterPosition") {
                        //if name.contains("GROUND") {
                        commands.entity(descendent).insert(FighterPosition);
                    }
                }
            }
        }
    }

    /// A setup function that that finds the descendent entity of each `Player` entity
    /// with a `FighterPosition` entity and inserts a `PositionEntity` component into
    /// the root entity.
    ///
    /// # Arguments
    /// * `commands` - Bevy commands struct,
    /// * `query` - A query to get all entities with a `FighterPosition` component
    /// * `parent_query` -  A query used to get all ancestors of the above query
    /// * `fighter_query` - A query to narrow down the above query to only those with `Fighter` and `Player` components
    ///
    ///
    ///
    pub fn set_position_entity(
        mut commands: Commands,
        query: Query<Entity, With<FighterPosition>>,
        parent_query: Query<&Parent>,
        fighter_query: Query<(With<Fighter>, With<Player>)>,
    ) {
        for entity in query.iter() {
            for fighter in parent_query.iter_ancestors(entity) {
                if fighter_query.get(fighter).is_ok() {
                    commands.entity(fighter).insert(PositionEntity(entity));
                }
            }
        }
    }

    /// A setup function that iterates over all entities with `HurtboxData` components, and finds their ancestor with
    /// a `Player` component. The root entity is referenced in the Hurtbox entity as an `Owner`, and the Hurtbox
    /// is added to the root's Hurtboxes list.
    ///
    /// # Panics
    ///
    /// Panics if the parent of the starting hurtbox entity does not have a parent entity,
    /// or if the root fighter entity does not contain a `Hurtboxes` component.
    ///
    ///
    /// # Note
    ///
    /// May loop indefinitely if the `HurtboxData` query never finds any entities
    ///
    pub fn armature_system(
        mut commands: Commands,
        hurtbox_query: Query<(Entity, &HurtboxData)>,
        parent_query: Query<&Parent>,
        mut state: ResMut<RoundState>,
        mut fighter_query: Query<&mut Hurtboxes>,
    ) {
        let hurt_iter = hurtbox_query.iter();

        if hurt_iter.len() > 0 {
            for (hurt_ent, _hurtbox) in hurt_iter {
                println!("How many time does this activate?");
                let mut ancestor = parent_query
                    .get(hurt_ent)
                    .expect("Entity doesn't have Parent");
                loop {
                    if let Ok(parent) = parent_query.get(ancestor.get()) {
                        ancestor = parent;
                    } else {
                        break;
                    }
                }

                commands.entity(hurt_ent).insert(Owner(ancestor.get()));

                let mut hurtboxes = fighter_query
                    .get_mut(ancestor.get())
                    .expect("Does not have Hurtboxes component");
                hurtboxes.add(hurt_ent);
            }

            *state = RoundState::EnterRound;
        }

        //println!("Does it stay here endlessly?");
    }

    pub fn add_animation_player_system(
        mut commands: Commands,
        anim_player_query: Query<(Entity, &AnimationPlayer)>,
        parent_query: Query<&Parent>,
    ) {
        for (anim, _) in anim_player_query.iter() {
            //println!("What about here?");
            if let Ok(parent) = parent_query.get(anim) {
                if let Ok(fighter) = parent_query.get(parent.get()) {
                    commands.entity(fighter.get()).insert(AnimEntity(anim));
                }
            }
        }
    }

    pub fn reparent_hurtbox_system(
        mut commands: Commands,
        players: Res<PlayerEntities>,
        mut data: ResMut<Assets<FullBoneTransformMap>>,
        handle_access: Res<PlayerHandleAccess>,
        hurtboxes_query: Query<&Hurtboxes>,
        parent_query: Query<(Entity, &Parent)>,
        name_query: Query<&Name>,
    ) {
        let bones1 = data
            .remove(&handle_access.0.bones)
            .expect("FullBoneTransformMap doesn't exist");
        // let bones2 = data
        //     .remove(&handle_access.1.bones)
        //     .expect("FullBoneTransformMap doesn't exist");

        let mut hurt_closure = |player: Entity, bones_trans: FullBoneTransformMap| {
            if let Ok(hurtboxes) = hurtboxes_query.get(player) {
                let mut children: Vec<Entity> = Vec::new();

                for (ent, parent) in parent_query.iter_many(hurtboxes.list()) {
                    let name = name_query
                        .get(parent.get())
                        .expect("Parent doesn't contain Name");

                    let trans_list = bones_trans
                        .0
                        .get(name.as_str())
                        .expect("FullBoneTransformMap doesn't contain bone name");

                    let converted: HashMap<String, Arc<Vec<Transform>>> = trans_list
                        .into_iter()
                        .map(|(action, list)| {
                            let mut trans: Vec<Transform> = Vec::new();
                            for matrix in list
                                .into_iter()
                                .map(|seq| Mat4::from_cols_array_2d(seq).transpose())
                            {
                                let mut temp = Transform::from_matrix(matrix);
                                trans.push(Transform::from_matrix(matrix));
                            }

                            (action.to_string(), Arc::new(trans))
                        })
                        .collect();

                    let child = commands
                        .spawn_empty()
                        .insert(TransformListRef(
                            converted.get("Idle").expect("No Idle in List").clone(),
                        ))
                        .insert(BoneTransforms(converted))
                        .insert(TransformBundle::default())
                        .insert(VisibilityBundle::default())
                        .add_child(ent)
                        .id();

                    children.push(child);
                }

                let hurt_transforms = commands
                    .spawn(SpatialBundle {
                        //transform: Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                        transform: Transform::from_rotation(Quat::from_euler(bevy::prelude::EulerRot::XYZ, -FRAC_PI_2, 0., PI)),
                        
                        ..default()
                    })
                    .insert(Name::new("Hurtbox Bones"))
                    .id();

                commands.entity(hurt_transforms).push_children(&children);
                commands.entity(player).add_child(hurt_transforms);
            }
        };

        hurt_closure(players.0, bones1);
        //hurt_closure(players.1, bones2);
    }
}

pub(crate) mod components {
    use std::sync::Arc;

    use bevy::{
        prelude::{AnimationClip, Component, Entity, Handle, Transform},
        reflect::{GetTypeRegistration, TypeRegistration, TypeUuid},
        utils::HashMap,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Component)]
    pub struct AnimEntity(pub Entity);

    #[derive(Component)]
    pub struct PositionEntity(pub Entity);

    #[derive(Component)]
    pub struct Animation(pub Handle<AnimationClip>, pub f32);

    impl Animation {
        pub fn length(&self) -> f32 {
            self.1
        }
    }

    #[derive(Debug, Component)]
    pub struct BoneTransforms(pub HashMap<String, Arc<Vec<Transform>>>);

    #[derive(Component)]
    pub struct TransformListRef(pub Arc<Vec<Transform>>);

    impl TransformListRef {
        pub fn get_frame(&self, frame: u16) -> usize {
            (frame as usize) % self.0.len()
        }
    }

    #[derive(Serialize, Deserialize, TypeUuid, Clone)]
    #[uuid = "9728bbfc-7beb-493a-b9a3-e4d63b5b0e81"]
    pub struct FullBoneTransformMap(pub HashMap<String, HashMap<String, Vec<[[f32; 4]; 4]>>>);
}

pub mod rollback {
    use crate::{fighter::state::StateMap, game::FRAME};

    use super::{super::state::State, components::Animation};

    use crate::fighter::Fighter;

    use bevy::prelude::{Changed, Children, HierarchyQueryExt, Ref, Transform, With};

    use super::super::state::{CurrentState, State as FightState, StateFrame};
    use super::components::{AnimEntity, BoneTransforms, FullBoneTransformMap, TransformListRef};

    use bevy::prelude::{AnimationPlayer, Commands, Entity, Parent, Query};

    /// Rollback System
    ///
    /// # Panics
    ///
    /// Panics if .
    #[allow(clippy::type_complexity)]
    pub fn animation_system(
        mut commands: Commands,
        mut animation_play: Query<(Entity, &Parent, &mut AnimationPlayer)>,
        fighter_query: Query<
            (
                Entity,
                Option<&AnimEntity>,
                &CurrentState,
                &StateMap,
                &StateFrame,
            ),
            With<Fighter>,
        >,
        parent_query: Query<&Parent>,
        state_query: Query<&Animation, With<State>>,
    ) {
        for (entity, anim_entity, current, _map, frame) in fighter_query.iter() {
            if let Some(anim) = anim_entity {
                if let Ok((_, _, mut player)) = animation_play.get_mut(anim.0) {
                    //let state = map.get(&current.0).expect("State doesn't exist");
                    if let Ok(animation) = state_query.get(current.0) {
                        player.play(animation.0.clone_weak());

                        player.set_elapsed((frame.0 as f32 * FRAME) % animation.length());
                        player.pause();
                    }
                }
            } else {
                for (play_ent, parent, mut _play) in animation_play.iter_mut() {
                    if let Ok(grandparent) = parent_query.get(parent.get()) {
                        if grandparent.get() == entity {
                            commands.entity(entity).insert(AnimEntity(play_ent));
                        }
                    }
                }
            }
        }
    }

    pub fn hurtbox_transform_system(
        query: Query<(Entity, &StateFrame), With<Fighter>>,
        children_query: Query<&Children>,
        mut transform_query: Query<(&mut Transform, &TransformListRef)>,
    ) {
        for (entity, frame) in query.iter() {
            for child in children_query.iter_descendants(entity) {
                if let Ok((mut tf, list_ref)) = transform_query.get_mut(child) {
                    //tf = list_ref.0.get(frame.0).expect("Transform doesn't exist for given frame");
                    *tf = list_ref
                        .0
                        .get(list_ref.get_frame(frame.0))
                        .expect("Transform doesn't exist for given frame")
                        .clone();
                }
            }
        }
    }

    pub fn on_change_state_system(
        query: Query<(&CurrentState, Entity), Changed<CurrentState>>,
        state_query: Query<&FightState>,
        children_query: Query<&Children>,
        mut child_query: Query<(&BoneTransforms, &mut TransformListRef)>,
    ) {
        for (current, entity) in query.iter() {
            let name = &state_query
                .get(current.0)
                .expect("Couldn't get State component")
                .name;

            for descendent in children_query.iter_descendants(entity) {
                if let Ok((bone_tfs, mut list_ref)) = child_query.get_mut(descendent) {
                    if let Some(r) = bone_tfs.0.get(name) {
                        list_ref.0 = r.clone();
                    }
                    
                    // list_ref.0 = bone_tfs
                    //     .0
                    //     .get(name)
                    //     .expect("List with given name doesn't exist in HashMap")
                    //     .clone();
                }
            }
        }
    }
}
