use bevy::prelude::*;
use fastrand::Rng;

pub struct RandomnessPlugin;

impl Plugin for RandomnessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Randomness>();
    }
}

#[derive(Resource, Default)]
pub struct Randomness {
    pub rng: Rng,
}
