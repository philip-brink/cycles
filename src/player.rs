use bevy::prelude::*;

use crate::track::TrackLaneId;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, highlight_player_path);
    }
}

pub struct Player {
    initial_lane: TrackLaneId,
}

fn highlight_player_path() {}
