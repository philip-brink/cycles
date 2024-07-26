use bevy::prelude::*;

use crate::bike::Bike;

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

trait Action {
    fn can_do(&self, bike: &Bike) -> bool;
}

struct Accelerate;

impl Action for Accelerate {
    fn can_do(&self, bike: &Bike) -> bool {
        todo!()
    }
}
