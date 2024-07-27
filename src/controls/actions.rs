use bevy::prelude::*;

use crate::{bike::Bike, RacingState};

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActionEvent>()
            .add_systems(Update, on_action.run_if(in_state(RacingState::Commanding)));
    }
}

#[derive(Component, PartialEq, Eq, Copy, Clone, Debug)]
pub enum ActionKind {
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

#[derive(Event, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ActionEvent(pub ActionKind);

trait Action {
    fn can_do(&self, bike: &Bike) -> bool;
}

struct Accelerate;

impl Action for Accelerate {
    fn can_do(&self, bike: &Bike) -> bool {
        todo!()
    }
}

fn on_action(mut action_events: EventReader<ActionEvent>) {
    for event in action_events.read() {
        println!("ACTION TRIGGERED: {event:?}");
    }
}
