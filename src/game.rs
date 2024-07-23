use bevy::prelude::*;

use crate::PlayingState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayingState::SetupRace), setup);
    }
}

fn setup() {
    // spawn track
}
