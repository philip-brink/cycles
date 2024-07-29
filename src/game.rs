mod finish_race;

use bevy::{prelude::*, time::Stopwatch};

use crate::{
    bike::Bike,
    collision::Collider,
    hud::HudPlugin,
    loading::{BikeTextures, TrackTexture},
    opponent::Opponent,
    player::Player,
    random::Randomness,
    track::{Track, TrackLaneId, TrackLanes, LAPS},
    GameState, PlayingState, RacingState,
};

use self::finish_race::FinishRacePlugin;

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
                (tick_turn_timer, update_laps, update_player_position)
                    .run_if(in_state(RacingState::Simulating)),
            )
            .add_systems(OnEnter(RacingState::Simulating), reset_timer)
            .add_systems(OnExit(GameState::Playing), teardown);
    }
}

#[derive(Component)]
struct Rider {
    laps: usize,
}

#[derive(Event)]
pub struct LapEvent(pub usize);

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
    q_track: Query<Entity, With<Track>>,
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
        Track,
        SpriteBundle {
            texture: track_texture.default.clone(),
            ..default()
        },
    ));
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
                Rider { laps: 0 },
                SpriteBundle {
                    texture: bike_textures.straight.clone(),
                    transform: Transform {
                        translation: position.extend(5.0),
                        ..default()
                    },
                    ..default()
                },
                Collider::new(120.0, 60.0),
            ))
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
    mut q_riders: Query<(&mut Rider, &Bike)>,
    track_lanes: Res<TrackLanes>,
    mut lap_event: EventWriter<LapEvent>,
    mut next_state: ResMut<NextState<PlayingState>>,
) {
    for (mut rider, bike) in q_riders.iter_mut() {
        let lane = track_lanes.track_lane(&bike.current_lane_id);
        let current_lap = lane.laps_finished(bike.distance);
        if rider.laps != current_lap {
            rider.laps = current_lap;
            lap_event.send(LapEvent(current_lap));
            if current_lap >= LAPS {
                next_state.set(PlayingState::FinishRace);
            }
        }
    }
}

fn update_player_position(
    q_opponents: Query<&Bike, With<Opponent>>,
    mut q_player: Query<(&Bike, &mut Player)>,
) {
    if let Ok((player_bike, mut player)) = q_player.get_single_mut() {
        let player_distance = player_bike.distance;
        let mut opponent_distances = Vec::new();
        for opponent_bike in &q_opponents {
            opponent_distances.push(opponent_bike.distance);
        }
        let mut player_pos = 4;
        for opponent_distance in opponent_distances {
            if player_distance > opponent_distance {
                player_pos -= 1;
            }
        }
        player.position = player_pos;
    }
}
