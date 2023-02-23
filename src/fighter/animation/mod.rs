pub mod setup {
    use crate::battle::HurtboxMaterial;
    use crate::fighter::data::Collider;
    use crate::AnimEntity;

    use bevy::gltf::GltfExtras;
    use bevy::prelude::AnimationPlayer;
    use bevy::prelude::Children;
    use bevy::prelude::HierarchyQueryExt;
    use parry3d::shape::Capsule;

    use super::super::state::Owner;

    use super::super::state::Hurtboxes;

    use crate::game::RoundState;

    use bevy::prelude::ResMut;

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

    use bevy::prelude::Assets;

    use bevy::prelude::Res;

    use bevy::prelude::Commands;

    /// A setup function that iterates over all entities with a StateMap, then iterates over
    /// that entity's state entities, inserting an Animation component in each
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

                    commands
                        .entity(entity)
                        .insert(Animation(animation.clone(), length));
                }
            }
        }
    }

    pub fn insert_hurtbox_data(
        mut commands: Commands,
        fighter_query: Query<Entity, With<Fighter>>,
        children_query: Query<&Children>,
        extras_query: Query<(&GltfExtras, &Name)>,

        hurtbox_material: Res<HurtboxMaterial>
    ) {
        let hurtbox_material = &hurtbox_material.0;

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
                }
            }
        }
    }

    /// A setup function that iterates over all entities with HurtboxData components, and finds their ancestor with
    /// a Player component. The root entity is referenced in the Hurtbox entity as an Owner, and the Hurtbox
    /// is added to the root's Hurtboxes list.
    ///
    /// # Panics
    ///
    /// Panics if the parent of the starting hurtbox entity does not have a parent entity,
    /// or if the root fighter entity does not contain a Hurtboxes component.
    ///
    ///
    /// # NOTE
    ///
    /// May loop indefinitely if the HurtboxData query never finds any entities
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

        println!("Does it stay here endlessly?");
    }

    pub fn add_animation_player_system(
        mut commands: Commands,
        anim_player_query: Query<(Entity, &AnimationPlayer)>,
        parent_query: Query<&Parent>,
    ) {
        for (anim, _) in anim_player_query.iter() {
            println!("What about here?");
            if let Ok(parent) = parent_query.get(anim) {
                if let Ok(fighter) = parent_query.get(parent.get()) {
                    commands.entity(fighter.get()).insert(AnimEntity(anim));
                }
            }
        }
    }
}

pub(crate) mod components {
    use bevy::prelude::{AnimationClip, Component, Handle};

    #[derive(Component)]
    pub struct Animation(pub Handle<AnimationClip>, pub f32);

    impl Animation {
        pub fn length(&self) -> f32 {
            self.1
        }
    }
}

pub(crate) mod rollback {
    use crate::{fighter::state::StateMap, game::FRAME};

    use super::{super::state::State, components::Animation};

    use super::super::super::Fighter;

    use bevy::prelude::With;

    use super::super::state::{CurrentState, StateFrame};

    use crate::AnimEntity;

    use bevy::prelude::{AnimationPlayer, Commands, Entity, Parent, Query};

    /// Rollback System
    ///
    /// # Panics
    ///
    /// Panics if .
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
        for (entity, anim_entity, current, map, frame) in fighter_query.iter() {
            if let Some(anim) = anim_entity {
                if let Ok((_, _, mut player)) = animation_play.get_mut(anim.0) {
                    let state = map.get(&current.0).expect("State doesn't exist");
                    if let Ok(animation) = state_query.get(*state) {
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
}
