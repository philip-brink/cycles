use bevy::prelude::*;

use crate::{loading::BikeTextures, track::TrackLane, PlayingState};

const SPEED: f32 = 600.0;
const TURNING_THRESHOLD: f32 = 0.00003;

pub struct BikePlugin;

impl Plugin for BikePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayingState::SetupRace), setup)
            .add_systems(
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
struct Bike {
    moving: bool,
    speed: f32,
    distance: f32,
    lane: TrackLane,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
enum BikeTurning {
    Left,
    Right,
}

fn setup(mut commands: Commands, bike_textures: Res<BikeTextures>) {
    let lanes = vec![50.0, 150.0, 250.0, 350.0];
    for lane_pos in lanes {
        let lane = TrackLane::new(lane_pos);
        let (pos, _) = lane.at_distance(0.0);
        commands.spawn((
            Bike {
                moving: false,
                speed: SPEED,
                distance: 0.0,
                lane: TrackLane::new(lane_pos),
            },
            SpriteBundle {
                texture: bike_textures.straight.clone(),
                transform: Transform {
                    translation: pos.extend(1.0),
                    ..default()
                },
                ..default()
            },
        ));
    }
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
