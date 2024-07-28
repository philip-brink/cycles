use bevy::{prelude::*, time::Stopwatch};

use crate::{
    bike::Bike,
    collision::Collider,
    loading::{BikeTextures, TrackTexture},
    opponent::Opponent,
    player::Player,
    random::Randomness,
    track::{TrackLaneId, TrackLanes},
    PlayingState, RacingState,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurnTimer>()
            .add_systems(
                OnEnter(PlayingState::SetupRace),
                (setup_track, setup_bikes).before(set_playing_state),
            )
            .add_systems(OnEnter(PlayingState::SetupRace), set_playing_state)
            .add_systems(
                Update,
                tick_turn_timer.run_if(in_state(RacingState::Simulating)),
            )
            .add_systems(OnEnter(RacingState::Simulating), reset_timer);
    }
}

#[derive(Resource)]
pub struct TurnTimer {
    timer: Timer,
    stopwatch: Stopwatch,
}

impl TurnTimer {
    pub fn proportion_finished(&self) -> f32 {
        self.stopwatch.elapsed_secs() / self.timer.duration().as_secs_f32()
    }
}

impl Default for TurnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            stopwatch: Stopwatch::new(),
        }
    }
}

fn tick_turn_timer(
    mut turn_timer: ResMut<TurnTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<RacingState>>,
) {
    turn_timer.timer.tick(time.delta());
    turn_timer.stopwatch.tick(time.delta());
    if turn_timer.timer.finished() {
        next_state.set(RacingState::Commanding);
    }
}

fn reset_timer(mut turn_timer: ResMut<TurnTimer>) {
    turn_timer.timer.reset();
    turn_timer.stopwatch.reset();
}

fn setup_track(mut commands: Commands, track_texture: Res<TrackTexture>) {
    commands.spawn(SpriteBundle {
        texture: track_texture.default.clone(),
        ..default()
    });
}

fn setup_bikes(
    mut commands: Commands,
    bike_textures: Res<BikeTextures>,
    track_lanes: Res<TrackLanes>,
    mut randomness: ResMut<Randomness>,
) {
    let lanes = [
        TrackLaneId::First,
        TrackLaneId::Second,
        TrackLaneId::Third,
        TrackLaneId::Fourth,
    ];
    let player_lane_index = randomness.rng.usize(..lanes.len());
    for (index, lane_id) in lanes.iter().enumerate() {
        let lane = track_lanes.track_lane(lane_id);
        let bike = Bike::new(lane_id, 1400.0, 0.5, 800.0);
        let (position, _) = lane.position_and_rotation(bike.distance);
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
                Collider::new(100.0, 200.0),
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
