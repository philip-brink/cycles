use bevy::prelude::*;

use crate::{PlayingState, RacingState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (toggle_simulating_state).run_if(in_state(PlayingState::Racing)),
        );
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
