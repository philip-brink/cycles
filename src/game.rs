mod finish_race;

use bevy::{prelude::*, time::Stopwatch};

use crate::{
    bike::{Bike, BikeBundle},
    collision::Collider,
    hud::HudPlugin,
    loading::{BikeTextures, TrackTexture},
    opponent::Opponent,
    player::Player,
    random::Randomness,
    track::{Track, TrackLaneId, TrackPosition, TrackVisual},
    GameState, PlayingState, RacingState,
};

use self::finish_race::FinishRacePlugin;

pub const LAPS: usize = 4;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurnTimer>()
            .add_plugins(HudPlugin)
            .add_plugins(FinishRacePlugin)
            .add_event::<LapEvent>()
            .add_systems(
                OnEnter(PlayingState::SetupRace),
                (setup_track, setup_bikes).before(set_playing_state),
            )
            .add_systems(OnEnter(PlayingState::SetupRace), set_playing_state)
            .add_systems(
                Update,
                (tick_turn_timer, update_laps).run_if(in_state(RacingState::Simulating)),
            )
            .add_systems(OnEnter(RacingState::Simulating), reset_timer)
            .add_systems(OnExit(GameState::Playing), teardown);
    }
}

#[derive(Event)]
pub struct LapEvent {
    pub entity: Entity,
    pub laps: usize,
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

fn teardown(
    mut commands: Commands,
    q_track: Query<Entity, With<TrackVisual>>,
    q_bikes: Query<Entity, With<Bike>>,
) {
    for entity in &q_track {
        commands.entity(entity).despawn_recursive();
    }

    for entity in &q_bikes {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_track(mut commands: Commands, track_texture: Res<TrackTexture>) {
    commands.spawn((
        TrackVisual,
        SpriteBundle {
            texture: track_texture.default.clone(),
            ..default()
        },
    ));
}

fn setup_bikes(
    mut commands: Commands,
    bike_textures: Res<BikeTextures>,
    track: Res<Track>,
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
        let max_speed = 1400.0;
        let acceleration = 800.0;
        let track_position = TrackPosition::new(*lane_id);
        let (position, rotation) = track_position.position_and_rotation(&track);
        let entity = commands
            .spawn(BikeBundle {
                bike: Bike::new(max_speed, acceleration, *lane_id),
                track_position,
                collider: Collider::new(120.0, 60.0),
                sprite_bundle: SpriteBundle {
                    texture: bike_textures.straight.clone(),
                    transform: Transform {
                        translation: position.extend(5.0),
                        rotation,
                        ..default()
                    },
                    ..default()
                },
            })
            .id();
        if player_lane_index == index {
            commands.entity(entity).insert(Player::new());
        } else {
            commands.entity(entity).insert(Opponent);
        };
    }
}

fn set_playing_state(mut next_state: ResMut<NextState<PlayingState>>) {
    next_state.set(PlayingState::Racing);
}

fn update_laps(
    mut q_bikes: Query<(Entity, &mut Bike, &TrackPosition), Changed<TrackPosition>>,
    track: Res<Track>,
    mut lap_events: EventWriter<LapEvent>,
) {
    for (entity, mut bike, track_position) in q_bikes.iter_mut() {
        let num_laps_complete = track.num_laps_completed(track_position.total_distance());
        if bike.laps != num_laps_complete {
            bike.laps = num_laps_complete;
            lap_events.send(LapEvent {
                entity,
                laps: num_laps_complete,
            });
        }
    }
}
