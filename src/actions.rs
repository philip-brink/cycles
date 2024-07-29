use bevy::prelude::*;

use crate::{
    bike::Bike,
    collision::{Collision, CollisionSide},
    track::TrackLaneId,
    RacingState,
};

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActionEvent>()
            .add_systems(Update, on_action.run_if(in_state(RacingState::Commanding)));
    }
}

#[derive(Component, PartialEq, Eq, Copy, Clone, Debug)]
pub enum BikeAction {
    Accelerate,
    Watch,
    Skid,
    Stop,
    Left,
    LeftLeft,
    LeftElbow,
    LeftHip,
    Right,
    RightRight,
    RightElbow,
    RightHip,
}

impl BikeAction {
    pub fn can_do(&self, bike: &Bike, maybe_collision: Option<&Collision>) -> bool {
        match self {
            BikeAction::Accelerate => match maybe_collision {
                Some(collision) => {
                    collision.side != CollisionSide::Front && bike.speed < bike.max_speed
                }
                None => bike.speed < bike.max_speed,
            },
            BikeAction::Watch => true,
            BikeAction::Skid => bike.speed > bike.max_speed / 2.0,
            BikeAction::Stop => bike.speed > 0.0,
            BikeAction::Left => {
                if bike.current_lane_id == TrackLaneId::First {
                    false
                } else if let Some(collision) = maybe_collision {
                    return collision.side != CollisionSide::Left;
                } else {
                    return true;
                }
            }
            BikeAction::LeftLeft => {
                if bike.current_lane_id == TrackLaneId::First
                    || bike.current_lane_id == TrackLaneId::Second
                {
                    false
                } else if let Some(collision) = maybe_collision {
                    return collision.side != CollisionSide::Left;
                } else {
                    return true;
                }
            }
            BikeAction::LeftElbow => bike.current_lane_id != TrackLaneId::First,
            BikeAction::LeftHip => bike.current_lane_id != TrackLaneId::First,
            BikeAction::Right => {
                if bike.current_lane_id == TrackLaneId::Fourth {
                    false
                } else if let Some(collision) = maybe_collision {
                    return collision.side != CollisionSide::Right;
                } else {
                    return true;
                }
            }
            BikeAction::RightRight => {
                if bike.current_lane_id == TrackLaneId::Fourth
                    || bike.current_lane_id == TrackLaneId::Third
                {
                    false
                } else if let Some(collision) = maybe_collision {
                    return collision.side != CollisionSide::Right;
                } else {
                    return true;
                }
            }
            BikeAction::RightElbow => bike.current_lane_id != TrackLaneId::Fourth,
            BikeAction::RightHip => bike.current_lane_id != TrackLaneId::Fourth,
        }
    }
}

#[derive(Event, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ActionEvent {
    pub bike_entity: Entity,
    pub kind: BikeAction,
}

impl ActionEvent {
    pub fn new(bike_entity: Entity, kind: BikeAction) -> Self {
        Self { bike_entity, kind }
    }
}

fn on_action(
    mut action_events: EventReader<ActionEvent>,
    mut commands: Commands,
    q_bikes: Query<(&Bike, Option<&Collision>)>,
    mut next_state: ResMut<NextState<RacingState>>,
) {
    for event in action_events.read() {
        if let Ok((bike, maybe_collision)) = q_bikes.get(event.bike_entity) {
            if event.kind.can_do(bike, maybe_collision) {
                commands.entity(event.bike_entity).insert(event.kind);
                println!(
                    "Doing action {:?} for bike {:?}",
                    event.kind, event.bike_entity,
                );
                next_state.set(RacingState::Simulating);
            }
        }
    }
}
