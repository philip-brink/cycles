use bevy::prelude::*;
use rand::Rng;

use crate::{
    bike::Bike,
    loading::{BikeTextures, TrackTexture},
    opponent::Opponent,
    player::Player,
    track::TrackLaneId,
    PlayingState,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PlayingState::SetupRace),
            (setup_track, setup_bikes).before(set_playing_state),
        )
        .add_systems(OnEnter(PlayingState::SetupRace), set_playing_state);
    }
}

fn setup_track(mut commands: Commands, track_texture: Res<TrackTexture>) {
    commands.spawn(SpriteBundle {
        texture: track_texture.default.clone(),
        ..default()
    });
}

fn setup_bikes(mut commands: Commands, bike_textures: Res<BikeTextures>) {
    let lanes = [
        TrackLaneId::First,
        TrackLaneId::Second,
        TrackLaneId::Third,
        TrackLaneId::Fourth,
    ];
    let mut rng = rand::thread_rng();
    let player_lane_index = rng.gen_range(0..lanes.len());
    for (index, lane_id) in lanes.iter().enumerate() {
        let bike = Bike::new(lane_id);
        let (position, _) = bike.position_and_direction();
        let entity = commands
            .spawn((
                bike,
                SpriteBundle {
                    texture: bike_textures.straight.clone(),
                    transform: Transform {
                        translation: position.extend(5.0),
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();
        if player_lane_index == index {
            commands.entity(entity).insert(Player);
        } else {
            commands.entity(entity).insert(Opponent);
        };
    }
}

fn set_playing_state(mut next_state: ResMut<NextState<PlayingState>>) {
    next_state.set(PlayingState::Racing);
}
