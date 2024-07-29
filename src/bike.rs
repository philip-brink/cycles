use bevy::prelude::*;

use crate::{
    actions::BikeAction,
    collision::{self, Collision},
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
            (on_turning_added, on_turning_removed, update_bikes_positions)
                .run_if(in_state(PlayingState::Racing)),
        )
        .add_systems(
            Update,
            (try_action, change_speed, on_collision, move_bikes)
                .chain()
                .run_if(in_state(RacingState::Simulating)),
        )
        .add_systems(OnEnter(RacingState::Simulating), check_slip)
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
    // pub grip: f32,
}

impl Bike {
    pub fn new(initial_lane: &TrackLaneId, max_speed: f32, _grip: f32, acceleration: f32) -> Self {
        Self {
            current_lane_id: *initial_lane,
            max_speed,
            acceleration,
            // grip,
            ..Default::default()
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
enum BikeTurning {
    Left,
    // Right,
}

fn try_action(
    q_bikes: Query<(
        Entity,
        &Bike,
        Option<&BikeAction>,
        Option<&ChangeSpeed>,
        Option<&ChangeLane>,
    )>,
    mut commands: Commands,
) {
    for (entity, bike, maybe_action, maybe_change_speed, maybe_change_lane) in q_bikes.iter() {
        if let Some(action) = maybe_action {
            match action {
                BikeAction::Accelerate => {
                    if maybe_change_speed.is_none() {
                        commands.entity(entity).insert(ChangeSpeed {
                            start_speed: bike.speed,
                            final_speed: (bike.speed + bike.acceleration).min(bike.max_speed),
                            instant: false,
                        });
                    }
                }
                BikeAction::Watch => {}
                BikeAction::Skid => {}
                BikeAction::Stop => {
                    if maybe_change_speed.is_none() {
                        commands.entity(entity).insert(ChangeSpeed {
                            start_speed: bike.speed,
                            final_speed: 0.0,
                            instant: false,
                        });
                    }
                }
                BikeAction::Left => {
                    if maybe_change_lane.is_none() {
                        commands.entity(entity).insert(ChangeLane::new(
                            bike.current_lane_id,
                            bike.current_lane_id.left(),
                        ));
                    }
                }
                BikeAction::LeftLeft => {
                    if maybe_change_lane.is_none() {
                        commands.entity(entity).insert(ChangeLane::new(
                            bike.current_lane_id,
                            bike.current_lane_id.left_left(),
                        ));
                    }
                }
                BikeAction::LeftElbow => {}
                BikeAction::LeftHip => {}
                BikeAction::Right => {
                    if maybe_change_lane.is_none() {
                        commands.entity(entity).insert(ChangeLane::new(
                            bike.current_lane_id,
                            bike.current_lane_id.right(),
                        ));
                    }
                }
                BikeAction::RightRight => {
                    if maybe_change_lane.is_none() {
                        commands.entity(entity).insert(ChangeLane::new(
                            bike.current_lane_id,
                            bike.current_lane_id.right_right(),
                        ));
                    }
                }
                BikeAction::RightElbow => {}
                BikeAction::RightHip => {}
            }
        }
    }
}

fn change_speed(mut q_bikes: Query<(&mut Bike, &ChangeSpeed)>, turn_timer: Res<TurnTimer>) {
    for (mut bike, change_speed) in q_bikes.iter_mut() {
        if change_speed.instant {
            bike.speed = change_speed.final_speed;
        } else {
            bike.speed = change_speed.current_speed(turn_timer.proportion_finished());
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct ChangeSpeed {
    start_speed: f32,
    final_speed: f32,
    instant: bool,
}

impl ChangeSpeed {
    fn current_speed(self, turn_proportion_elapsed: f32) -> f32 {
        self.start_speed + (self.final_speed - self.start_speed) * turn_proportion_elapsed
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct ChangeLane {
    start_lane_id: TrackLaneId,
    final_lane_id: TrackLaneId,
    double_lane_change: bool,
    current_proportion: f32,
    lane_clear: bool,
    changing_to_left: bool,
}

impl ChangeLane {
    fn new(current: TrackLaneId, desired: TrackLaneId) -> Self {
        let double_lane_change = current.difference(desired) > 1;
        Self {
            start_lane_id: current,
            final_lane_id: desired,
            double_lane_change,
            current_proportion: 0.0,
            lane_clear: true,
            changing_to_left: current.is_to_right_of(desired),
        }
    }
    fn update_proportion(&mut self, turn_proportion_elapsed: f32) {
        if self.lane_clear {
            self.current_proportion = 0.0.lerp(1.0, turn_proportion_elapsed);
        } else {
            self.current_proportion = self.current_proportion.lerp(0.0, turn_proportion_elapsed);
        }
    }
    fn final_lane(&self) -> TrackLaneId {
        if self.double_lane_change {
            if self.current_proportion < 0.4 {
                self.start_lane_id
            } else if self.current_proportion > 0.9 {
                return self.final_lane_id;
            } else {
                return self.start_lane_id.between(self.final_lane_id);
            }
        } else if self.current_proportion < 0.6 {
            return self.start_lane_id;
        } else {
            return self.final_lane_id;
        }
    }
}

fn move_bikes(
    mut q_bikes: Query<(&mut Bike, Option<&mut ChangeLane>)>,
    time: Res<Time>,
    turn_timer: Res<TurnTimer>,
) {
    for (mut bike, maybe_change_lane) in q_bikes.iter_mut() {
        bike.distance += bike.speed * time.delta_seconds();
        if let Some(mut change_lane) = maybe_change_lane {
            change_lane.update_proportion(turn_timer.proportion_finished());
        }
    }
}

fn update_bikes_positions(
    mut q_bike: Query<(
        Entity,
        &Bike,
        &mut Transform,
        Option<&BikeTurning>,
        Option<&ChangeLane>,
    )>,
    lanes: Res<TrackLanes>,
    mut commands: Commands,
) {
    for (entity, bike, mut transform, maybe_turning, maybe_changing_lane) in q_bike.iter_mut() {
        let (pos, rot) = match maybe_changing_lane {
            Some(change_lane) => lanes.pos_and_rot_between_lanes(
                change_lane.start_lane_id,
                change_lane.final_lane_id,
                bike.distance,
                change_lane.current_proportion,
            ),
            None => lanes.pos_and_rot_between_lanes(
                bike.current_lane_id,
                bike.current_lane_id,
                bike.distance,
                0.0,
            ),
        };
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

fn on_collision(
    mut q_bike_collisions: Query<
        (Entity, &Bike, &Collision, Option<&mut ChangeLane>),
        Added<Collision>,
    >,
    mut commands: Commands,
) {
    for (entity, bike, collision, maybe_change_lane) in q_bike_collisions.iter_mut() {
        match collision.side {
            collision::CollisionSide::Front => {
                // slow down to other bike's speed
                commands.entity(entity).insert(ChangeSpeed {
                    start_speed: bike.speed,
                    final_speed: collision.other_bike_speed,
                    instant: true,
                });
                let speed_difference = (bike.speed - collision.other_bike_speed).abs();
                if speed_difference > 10.0 {
                    println!("CRASH!!!");
                }
            }
            collision::CollisionSide::Left => {
                if let Some(mut change_lane) = maybe_change_lane {
                    if change_lane.changing_to_left {
                        println!("Blocked!");
                        change_lane.lane_clear = false;
                    }
                }
            }
            collision::CollisionSide::Right => {
                if let Some(mut change_lane) = maybe_change_lane {
                    if !change_lane.changing_to_left {
                        println!("Blocked!");
                        change_lane.lane_clear = false;
                    }
                }
            }
            collision::CollisionSide::Back => {
                // do nothing
            }
        }
    }
}

fn check_slip(
    q_bike: Query<(Entity, &Bike, Option<&BikeAction>)>,
    track_lanes: Res<TrackLanes>,
    mut commands: Commands,
) {
    for (entity, bike, maybe_bike_action) in &q_bike {
        if let Some(bike_action) = maybe_bike_action {
            if *bike_action == BikeAction::Skid {
                return;
            }
        }
        if track_lanes
            .track_lane(&bike.current_lane_id)
            .in_turn(bike.distance)
        {
            let max_turn_speed = ((4 - bike.current_lane_id as i32) * 400) as f32;
            if bike.speed > max_turn_speed {
                let final_lane_id = if bike.speed - max_turn_speed > 800.0 {
                    println!("SLIP DOUBLE");
                    bike.current_lane_id.right_right()
                } else {
                    println!("SLIP");
                    bike.current_lane_id.right()
                };
                commands
                    .entity(entity)
                    .insert(ChangeLane::new(bike.current_lane_id, final_lane_id));
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
            // BikeTurning::Right => *image_handle = bike_textures.turn.clone(),
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
    mut q_bikes: Query<(Entity, &mut Bike, Option<&ChangeLane>)>,
    q_actions: Query<Entity, With<BikeAction>>,
    mut commands: Commands,
    track_lanes: Res<TrackLanes>,
) {
    for (entity, mut bike, maybe_change_lane) in q_bikes.iter_mut() {
        if let Some(change_lane) = maybe_change_lane {
            let new_lane_distance = track_lanes.distance_on_adjacent_lane(
                bike.current_lane_id,
                change_lane.final_lane(),
                bike.distance,
            );
            bike.distance = new_lane_distance;
            bike.current_lane_id = change_lane.final_lane();
            commands.entity(entity).remove::<ChangeLane>();
        }
    }
    for entity in &q_actions {
        commands.entity(entity).remove::<BikeAction>();
        commands.entity(entity).remove::<ChangeSpeed>();
    }
}
