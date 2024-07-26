use bevy::prelude::*;

use crate::{
    loading::BikeTextures,
    track::{TrackLane, TrackLaneId},
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
            (update_bikes).run_if(in_state(RacingState::Simulating)),
        )
        .add_systems(OnEnter(RacingState::Commanding), on_enter_commanding_state)
        .add_systems(OnEnter(RacingState::Simulating), on_enter_simulating_state);
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Bike {
    pub lane_id: TrackLaneId,
    pub distance: f32,
    pub moving: bool,
    speed: f32,
    pub lane: TrackLane,
}

impl Bike {
    pub fn new(initial_lane: &TrackLaneId) -> Self {
        let lane = TrackLane::new(initial_lane);
        let distance = 0.0;
        Self {
            lane_id: *initial_lane,
            distance,
            moving: false,
            speed: 600.0,
            lane,
        }
    }

    pub fn position_and_direction(&self) -> (Vec2, Quat) {
        self.lane.position_and_rotation(self.distance)
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
enum BikeTurning {
    Left,
    Right,
}

fn update_bikes(
    mut q_bike: Query<(Entity, &mut Bike, &mut Transform, Option<&BikeTurning>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut bike, mut transform, maybe_turning) in q_bike.iter_mut() {
        if bike.moving {
            bike.distance += bike.speed * time.delta_seconds();
            let (pos, rot) = bike.lane.position_and_rotation(bike.distance);
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

fn on_enter_simulating_state(mut q_bikes: Query<&mut Bike>) {
    for mut bike in q_bikes.iter_mut() {
        bike.moving = true;
    }
}

fn on_enter_commanding_state(mut q_bikes: Query<&mut Bike>) {
    for mut bike in q_bikes.iter_mut() {
        bike.moving = false;
    }
}
