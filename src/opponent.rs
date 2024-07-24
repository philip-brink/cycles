use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, _app: &mut App) {
        // todo
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Opponent;
