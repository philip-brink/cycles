use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, highlight_player_path);
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Player;

fn highlight_player_path() {}