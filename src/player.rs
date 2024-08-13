use bevy::prelude::*;

use crate::{
    bike::Bike,
    opponent::Opponent,
    track::{Track, TrackPosition},
    PlayingState, RacingState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerPositionEvent>().add_systems(
            Update,
            (toggle_simulating_state, update_player_position)
                .run_if(in_state(PlayingState::Racing)),
        );
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Player {
    pub position: usize,
}

impl Player {
    pub fn new() -> Self {
        Self { position: 4 }
    }
}

#[derive(Event)]
pub struct PlayerPositionEvent(pub usize);

fn update_player_position(
    q_opponents: Query<&TrackPosition, (With<Opponent>, With<Bike>)>,
    mut q_player: Query<(&TrackPosition, &mut Player), With<Bike>>,
    mut player_position_event: EventWriter<PlayerPositionEvent>,
    track: Res<Track>,
) {
    if let Ok((player_pos, mut player)) = q_player.get_single_mut() {
        let player_distance = player_pos.total_distance(&track);
        let opponent_distances = q_opponents.iter().map(|e| e.total_distance(&track));
        let mut player_pos = 4;
        for opponent_distance in opponent_distances {
            if player_distance > opponent_distance {
                player_pos -= 1;
            }
        }
        if player.position != player_pos {
            player.position = player_pos;
            player_position_event.send(PlayerPositionEvent(player_pos));
        }
    }
}

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
        }
    }
}
