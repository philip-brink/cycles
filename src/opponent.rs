use bevy::prelude::*;

use crate::{
    actions::{self, BikeAction},
    bike::Bike,
    collision::Collision,
    random::Randomness,
    RacingState,
};

const BIKE_ACTIONS: [actions::BikeAction; 12] = [
    BikeAction::Accelerate,
    BikeAction::Watch,
    BikeAction::Skid,
    BikeAction::Stop,
    BikeAction::Left,
    BikeAction::LeftLeft,
    BikeAction::LeftElbow,
    BikeAction::LeftHip,
    BikeAction::Right,
    BikeAction::RightRight,
    BikeAction::RightElbow,
    BikeAction::RightHip,
];

pub struct OpponentPlugin;

impl Plugin for OpponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RacingState::Commanding), act);
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Opponent;

fn act(
    q_opponents: Query<(Entity, &Bike, Option<&Collision>), With<Opponent>>,
    mut commands: Commands,
    mut randomness: ResMut<Randomness>,
) {
    for (entity, bike, maybe_collision) in &q_opponents {
        if BikeAction::Accelerate.can_do(bike, maybe_collision) {
            commands.entity(entity).insert(BikeAction::Accelerate);
        } else {
            let possible_actions = generate_possible_actions(bike, maybe_collision);
            let random_action_index = randomness.rng.usize(0..possible_actions.len());
            commands
                .entity(entity)
                .insert(possible_actions[random_action_index]);
        }
    }
}

fn generate_possible_actions(bike: &Bike, maybe_collision: Option<&Collision>) -> Vec<BikeAction> {
    BIKE_ACTIONS
        .iter()
        .filter(|e| e.can_do(bike, maybe_collision))
        .copied()
        .collect::<Vec<BikeAction>>()
}
