use bevy::prelude::*;

use crate::{
    loading::{BikeTextures, TrackTexture},
    PlayingState,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlayingState::SetupRace),
            (setup_track, setup_player, setup_opponents),
        );
    }
}

fn setup_track(mut commands: Commands, track_texture: Res<TrackTexture>) {
    commands.spawn(SpriteBundle {
        texture: track_texture.default.clone(),
        ..default()
    });
}

fn setup_player(mut commands: Commands, track_texture: Res<BikeTextures>) {}

fn setup_opponents(mut commands: Commands, track_texture: Res<BikeTextures>) {}
