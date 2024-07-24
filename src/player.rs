use bevy::prelude::*;

use crate::{
    bike::Bike,
    path_highlight::{HidePathHighlightEvent, ShowPathHighlightEvent},
    PlayingState, RacingState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (toggle_simulating_state).run_if(in_state(PlayingState::Racing)),
        )
        .add_systems(OnEnter(RacingState::Simulating), on_enter_simulating_state)
        .add_systems(OnEnter(RacingState::Commanding), on_enter_commanding_state);
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Player;

fn toggle_simulating_state(
    mut next_state: ResMut<NextState<RacingState>>,
    state: Res<State<RacingState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::Enter]) {
        match state.get() {
            RacingState::Simulating => {
                next_state.set(RacingState::Commanding);
            }
            RacingState::Commanding => {
                next_state.set(RacingState::Simulating);
            }
            RacingState::Paused => {
                // do nothing
            }
        }
    }
}

fn on_enter_simulating_state(
    mut hide_path_event_writer: EventWriter<HidePathHighlightEvent>,
    mut q_bikes: Query<&mut Bike>,
) {
    for mut bike in q_bikes.iter_mut() {
        bike.moving = true;
    }
    hide_path_event_writer.send_default();
}

fn on_enter_commanding_state(
    mut show_path_event_writer: EventWriter<ShowPathHighlightEvent>,
    mut q_bikes: Query<&mut Bike>,
) {
    for mut bike in q_bikes.iter_mut() {
        bike.moving = false;
    }
    show_path_event_writer.send_default();
}
