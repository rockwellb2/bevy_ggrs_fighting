use bevy::{
    prelude::{Changed, Name, Query, With, Res},
    text::Text,
};

use crate::{
    battle::{ChangeText, PlayerEntities},
    fighter::state::{CurrentState, State as FightState, StateMap},
};

pub fn state_text_system(
    players: Option<Res<PlayerEntities>>,
    query: Query<(&CurrentState, &StateMap), Changed<CurrentState>>,
    state_query: Query<&Name>,
    mut text_query: Query<&mut Text, With<ChangeText>>,
) {
    if let Some(players) = players {
        if let Ok((current, map)) = query.get(players.get(1)) {
            //let s = map.get(&current.0).expect("State doesn't exist");

            let name = state_query.get(current.0).expect("A state has no name");

            if let Ok(mut text) = text_query.get_single_mut() {
                text.sections[1].value = name.to_string();
            }
        }
    }
}
