use bevy::prelude::*;

use crate::{
    loading::BikeTextures,
    track::{TrackLane, TrackLaneId},
};

const TURNING_THRESHOLD: f32 = 0.00003;

pub struct BikePlugin;

impl Plugin for BikePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_bikes,
                toggle_moving,
                on_turning_added,
                on_turning_removed,
            ),
        );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Bike {
    moving: bool,
    speed: f32,
    distance: f32,
    lane: TrackLane,
}

impl Bike {
    pub fn new(initial_lane: &TrackLaneId) -> Self {
        let lane = TrackLane::new(initial_lane);
        let distance = 0.0;
        Self {
            moving: false,
            speed: 600.0,
            distance,
            lane,
        }
    }

    pub fn position_and_direction(&self) -> (Vec2, Quat) {
        self.lane.at_distance(self.distance)
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
            let (pos, rot) = bike.lane.at_distance(bike.distance);
            transform.translation = pos.extend(1.0);
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

fn toggle_moving(
    mut q_bike: Query<(Entity, &mut Bike)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    if keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::Enter]) {
        for (entity, mut bike) in q_bike.iter_mut() {
            if bike.moving {
                commands.entity(entity).remove::<BikeTurning>();
            }
            bike.moving = !bike.moving;
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
