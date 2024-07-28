use bevy::prelude::*;

use crate::{bike::Bike, track::TrackLaneId, RacingState};

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
    pub fn can_do(&self, bike: &Bike) -> bool {
        match self {
            BikeAction::Accelerate => bike.speed < bike.max_speed,
            BikeAction::Watch => true,
            BikeAction::Skid => bike.speed > bike.max_speed / 2.0,
            BikeAction::Stop => bike.speed > 0.0,
            BikeAction::Left => bike.current_lane_id != TrackLaneId::First,
            BikeAction::LeftLeft => {
                bike.current_lane_id != TrackLaneId::First
                    && bike.current_lane_id != TrackLaneId::Second
            }
            BikeAction::LeftElbow => bike.current_lane_id != TrackLaneId::First,
            BikeAction::LeftHip => bike.current_lane_id != TrackLaneId::First,
            BikeAction::Right => bike.current_lane_id != TrackLaneId::Fourth,
            BikeAction::RightRight => bike.current_lane_id != TrackLaneId::Fourth,
            BikeAction::RightElbow => bike.current_lane_id != TrackLaneId::Fourth,
            BikeAction::RightHip => bike.current_lane_id != TrackLaneId::Fourth,
        }
    }

    pub fn is_repeated(&self) -> bool {
        match self {
            BikeAction::Accelerate => todo!(),
            BikeAction::Watch => todo!(),
            BikeAction::Skid => todo!(),
            BikeAction::Stop => todo!(),
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
    q_bikes: Query<&Bike>,
    mut next_state: ResMut<NextState<RacingState>>,
) {
    for event in action_events.read() {
        if let Ok(bike) = q_bikes.get(event.bike_entity) {
            if event.kind.can_do(bike) {
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
