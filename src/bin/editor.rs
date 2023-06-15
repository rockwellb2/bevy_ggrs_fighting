use bevy::app::CoreSchedule::FixedUpdate;
use bevy::utils::HashMap;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_editor_pls::EditorPlugin;
use bevy_fighting_lib::battle::{PlayerEntities, PlayerHandleAccess};
use bevy_fighting_lib::fighter::event::TransitionEvent;
use bevy_fighting_lib::fighter::state::{CurrentState, State as FightState, StateMap};
use bevy_fighting_lib::fighter::systems::{hitbox_component_system, hitbox_removal_system};
use bevy_fighting_lib::game::on_round;
use bevy_fighting_lib::{
    fighter::{animation::rollback::animation_system, state::StateFrame, FighterPlugin},
    game::{RoundState, SetupPlugin},
};
use bevy_inspector_egui::egui::panel::Side;
use bevy_inspector_egui::egui::Sense;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::{egui, prelude::*};

//use bevy_egui::EguiContext;
use bevy_inspector_egui::bevy_egui::EguiContext;
use serde::{Serialize, Deserialize};
use serde_pickle::{DeOptions, HashableValue};
use std::collections::BTreeMap;

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
struct PlayAnimation(bool);

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
struct StateListUI(Vec<(String, u16)>);

#[derive(Serialize, Deserialize, Debug)]
struct PickleMat([[f32; 4]; 4]);



fn main() {
    let mut app = App::new();

    let file = include_bytes!("../../assets/data/fighters/ryo/ryo.hurt");

    //let thing: HashMap<String, HashMap<String, Vec<serde_pickle::Value>>>= serde_pickle::from_slice(file, DeOptions::new().replace_unresolved_globals()).expect("Couldn't deserialize from Pickle");
    let thing: HashMap<String, HashMap<String, Vec<[[f32; 4]; 4]>>> = serde_pickle::from_slice(file, DeOptions::new().replace_unresolved_globals()).expect("Couldn't deserialize from Pickle");

    // let y = thing.get("spine middle").expect("Doesn't exist");
    
    // let v = &y["5A"];
    // let value = &v[1];
    // let x: [[f32; 4]; 4] = serde_pickle::from_value(value.clone()).expect("Couldn't convert");
    // let z = Mat4::from_cols_array_2d(&x);
    // let z = z.transpose();

    




    //println!("{:?}", z);


    app.add_plugin(SetupPlugin)
        .insert_resource(RoundState::EnterLoading)
        .insert_resource(PlayAnimation::default())
        .insert_resource(StateListUI::default())
        .register_type::<PlayAnimation>()
        .add_plugin(FighterPlugin)
        .add_plugin(EditorPlugin::default())
        .edit_schedule(FixedUpdate, |schedule| {
            schedule.add_system(frame_tick_system);
        })    
        .insert_resource(FixedTime::new_from_secs(1. / 60.))
        .add_system(inspector_ui)
        .add_system(animation_system.run_if(on_round))
        .add_system(hitbox_component_system.after(animation_system).run_if(on_round).run_if(on_state_frame_change))
        .add_system(hitbox_removal_system.run_if(on_round))
        .add_system(on_play_change_system.run_if(on_round))
        .add_system(on_round_state_change.run_if(on_round))
        .add_system(state_list_inspector)
        .add_system(player_inspector.run_if(on_round));

    app.run();
}

fn inspector_ui(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    let mut change: Option<i16> = None;

    egui::Window::new("Play").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                let minus = ui.button("-").clicked();
                bevy_inspector_egui::bevy_inspector::ui_for_resource::<PlayAnimation>(world, ui);
                let plus = ui.button("+").clicked();

                if minus || plus {
                    change = Some(if minus { -1 } else { 1 });
                }
            });
        });
    });

    if let Some(change) = change {
        let mut query = world.query::<&mut StateFrame>();

        for mut frame in query.iter_mut(world) {
            if change.is_negative() {
                frame.0 = frame.0.checked_sub(1).unwrap_or(0);
            }
            else {
                frame.0 += 1;
            }

        }
         
    }
}

fn player_inspector(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    let fighter = world.resource::<PlayerEntities>().0;

    egui::SidePanel::new(Side::Right, "Fighter").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            bevy_inspector_egui::bevy_inspector::ui_for_entity(world, fighter, ui);
        });
    });
}

fn on_state_frame_change(
    players: Option<Res<PlayerEntities>>,
    query: Query<Ref<StateFrame>>
) -> bool {
    if players.is_none() {
        return false;
    }

    query.get(players.expect("PlayerEntities resource does not exist").0).expect("Player entity doesn't have StateFrame").is_changed()
}

fn on_round_state_change(
    mut commands: Commands,
    state: Res<RoundState>,
    players: Res<PlayerEntities>,
    mut list: ResMut<StateListUI>,
    fighter_query: Query<&StateMap>,
    states_query: Query<(&Name, &FightState)>,
) {
    if state.is_changed() {
        match *state {
            RoundState::Round => {
                let map = fighter_query
                    .get(players.0)
                    .expect("StateMap for Player doesn't exist");

                let mut states: Vec<(String, u16)> = states_query
                    .iter_many(map.map.values())
                    .map(|(name, s)| {
                        (
                            name.split_once(' ')
                                .expect("Splitting string didn't work")
                                .1
                                .to_string(),
                            s.id,
                        )
                    })
                    .collect();

                states.sort_by(|(_, a), (_, b)| a.cmp(b));
                list.0 = states;
            }
            _ => return,
        }
    }
}

fn state_list_inspector(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    let mut transition: Option<u16> = None;

    egui::SidePanel::new(Side::Left, "States").show(egui_context.get_mut(), |ui| {
        ui.heading("States");
        egui::ScrollArea::vertical().show(ui, |ui| {
            //bevy_inspector_egui::bevy_inspector::ui_for_resource::<StateListUI>(world, ui);

            let list = world.resource::<StateListUI>();
            egui::Grid::new("List")
                .max_col_width(100.)
                .min_row_height(25.)
                .striped(true)
                .show(ui, |ui| {
                    for (name, id) in &list.0 {
                        ui.label(name);
                        ui.label(id.to_string());
                        if ui.button("Set").clicked() {
                            transition = Some(*id);
                        }
                        ui.end_row();
                    }
                });
            //ui.button("Something");
        });
    });

    if let Some(id) = transition {
        let mut query = world.query::<(&mut CurrentState, &mut StateFrame, &StateMap)>();

        for (mut s, mut frame, map) in query.iter_mut(world) {
            s.0 = *map.get(&id).expect("State with given ID doesn't exist");
            frame.0 = 0;
        }
    }
}

fn frame_tick_system(play_anim: Res<PlayAnimation>, mut frame_query: Query<&mut StateFrame>) {
    if play_anim.0 {
        for mut frame in frame_query.iter_mut() {
            frame.0 += 1;
        }
    }
}

fn on_play_change_system(play_anim: Res<PlayAnimation>, mut frame_query: Query<&mut StateFrame>) {
    if play_anim.is_changed() && !play_anim.0 {
        for mut frame in frame_query.iter_mut() {
            frame.0 = 0;
        }
    }
}
