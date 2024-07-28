use bevy::prelude::*;

use crate::{
    actions::BikeAction,
    game::TurnTimer,
    loading::BikeTextures,
    track::{TrackLaneId, TrackLanes},
    PlayingState, RacingState,
};

const TURNING_THRESHOLD: f32 = 0.00003;

pub struct BikePlugin;

impl Plugin for BikePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (on_turning_added, on_turning_removed).run_if(in_state(PlayingState::Racing)),
        )
        .add_systems(
            Update,
            (try_action, update_bikes)
                .chain()
                .run_if(in_state(RacingState::Simulating)),
        )
        .add_systems(OnEnter(RacingState::Commanding), on_enter_commanding_state)
        .add_systems(OnExit(RacingState::Simulating), on_exit_simulating_state);
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Bike {
    pub current_lane_id: TrackLaneId,
    pub distance: f32,
    pub speed: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub grip: f32,
}

impl Bike {
    pub fn new(initial_lane: &TrackLaneId, max_speed: f32, grip: f32, acceleration: f32) -> Self {
        Self {
            current_lane_id: *initial_lane,
            max_speed,
            acceleration,
            grip,
            ..Default::default()
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
enum BikeTurning {
    Left,
    Right,
}

fn try_action(mut q_bikes: Query<(&mut Bike, Option<&BikeAction>)>, turn_timer: Res<TurnTimer>) {
    for (mut bike, maybe_action) in q_bikes.iter_mut() {
        if let Some(action) = maybe_action {
            match action {
                BikeAction::Accelerate => {
                    bike.speed = (bike.speed
                        + (bike.acceleration * turn_timer.proportion_finished()))
                    .min(bike.max_speed);
                    println!("BIKE SPEED: {}", bike.speed);
                }
                BikeAction::Watch => {}
                BikeAction::Skid => todo!(),
                BikeAction::Stop => bike.speed = 0.0,
                BikeAction::Left => todo!(),
                BikeAction::LeftLeft => todo!(),
                BikeAction::LeftElbow => todo!(),
                BikeAction::LeftHip => todo!(),
                BikeAction::Right => todo!(),
                BikeAction::RightRight => todo!(),
                BikeAction::RightElbow => todo!(),
                BikeAction::RightHip => todo!(),
            }
        }
    }
}

fn update_bikes(
    mut q_bike: Query<(
        Entity,
        &mut Bike,
        &mut Transform,
        Option<&BikeTurning>,
        Option<&BikeAction>,
    )>,
    time: Res<Time>,
    turn_timer: Res<TurnTimer>,
    lanes: Res<TrackLanes>,
    mut commands: Commands,
) {
    for (entity, mut bike, mut transform, maybe_turning, maybe_action) in q_bike.iter_mut() {
        let current_lane = lanes.track_lane(&bike.current_lane_id);
        bike.distance += bike.speed * time.delta_seconds();
        let (pos, rot) = current_lane.position_and_rotation(bike.distance);
        transform.translation = pos.extend(5.0);
        let turning = (transform.rotation - rot).length_squared() > TURNING_THRESHOLD;
        transform.rotation = rot;
        if turning && maybe_turning.is_none() {
            commands.entity(entity).insert(BikeTurning::Left);
        } else if !turning && maybe_turning.is_some() {
            commands.entity(entity).remove::<BikeTurning>();
        }
    }
}

fn on_turning_added(
    mut q_bike: Query<(&BikeTurning, &mut Handle<Image>), Added<BikeTurning>>,
    bike_textures: Res<BikeTextures>,
) {
    for (turning, mut image_handle) in q_bike.iter_mut() {
        match turning {
            BikeTurning::Left => *image_handle = bike_textures.turn.clone(),
            BikeTurning::Right => *image_handle = bike_textures.turn.clone(),
        }
    }
}

fn on_turning_removed(
    mut removed_turning: RemovedComponents<BikeTurning>,
    mut q_bike: Query<&mut Handle<Image>, With<Bike>>,
    bike_textures: Res<BikeTextures>,
) {
    for entity in removed_turning.read() {
        if let Ok(mut image_handle) = q_bike.get_mut(entity) {
            *image_handle = bike_textures.straight.clone();
        }
    }
}

fn on_exit_simulating_state(
    mut q_bikes: Query<&mut Bike>,
    q_actions: Query<Entity, With<BikeAction>>,
    mut commands: Commands,
) {
    for mut bike in q_bikes.iter_mut() {
        //
    }
    for entity in &q_actions {
        commands.entity(entity).remove::<BikeAction>();
    }
}

fn on_enter_commanding_state(mut q_bikes: Query<&mut Bike>) {
    for mut bike in q_bikes.iter_mut() {
        //
    }
}
