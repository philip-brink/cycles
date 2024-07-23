use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{loading::TrackTexture, PlayingState};

const LAPS: i32 = 4;
const STRAIGHT_DISTANCE: f32 = 2000.0;
const TURN_RADIUS: f32 = 620.0;
const LANE_WIDTH: f32 = 100.0;

pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayingState::SetupRace), setup);
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TrackLaneId {
    /// Inner track
    #[default]
    First,
    /// Next to innermost track
    Second,
    /// Next to outermost track
    Third,
    /// Outer track
    Fourth,
}

impl TrackLaneId {
    fn distance_from_inner_edge(&self) -> f32 {
        let factor = match self {
            TrackLaneId::First => 1,
            TrackLaneId::Second => 2,
            TrackLaneId::Third => 3,
            TrackLaneId::Fourth => 4,
        };
        (LANE_WIDTH / 2.0) + (LANE_WIDTH * factor as f32)
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TrackLane {
    lap_distance: f32,
    half_straight_dist: f32,
    turn_radius: f32,
    vertical_offset: f32,
    first_straightaway_p1_dist: f32,
    first_turn_dist: f32,
    second_straightaway_dist: f32,
    second_turn_dist: f32,
}

impl TrackLane {
    pub fn new(length_from_inner_edge: f32) -> Self {
        let semicircle_circumfrence = PI * (TURN_RADIUS + length_from_inner_edge);
        let lap_distance = (STRAIGHT_DISTANCE + semicircle_circumfrence) * 2.0;
        let half_straight_dist = STRAIGHT_DISTANCE / 2.0;
        let turn_radius = TURN_RADIUS + length_from_inner_edge;
        let vertical_offset = turn_radius;
        let first_straightaway_p1_dist = half_straight_dist;
        let first_turn_dist = first_straightaway_p1_dist + semicircle_circumfrence;
        let second_straightaway_dist = first_turn_dist + STRAIGHT_DISTANCE;
        let second_turn_dist = second_straightaway_dist + semicircle_circumfrence;
        TrackLane {
            lap_distance,
            half_straight_dist,
            turn_radius,
            vertical_offset,
            first_straightaway_p1_dist,
            first_turn_dist,
            second_straightaway_dist,
            second_turn_dist,
        }
    }

    /// Determine the position and rotation at a specified distance
    /// from the starting position of 0.0.
    pub fn at_distance(&self, distance: f32) -> (Vec2, Quat) {
        let current_lap_distance = distance % self.lap_distance;
        if current_lap_distance <= self.first_straightaway_p1_dist {
            // in first straightaway (after finish line)
            let horizontal = current_lap_distance;
            let vertical = -self.vertical_offset;
            let rot = Quat::from_rotation_z(0.0);
            (Vec2::new(horizontal, vertical), rot)
        } else if current_lap_distance <= self.first_turn_dist {
            // in first turn
            let circle_dist = current_lap_distance - self.first_straightaway_p1_dist;
            let position_angle_offset = circle_dist / self.turn_radius;
            let position_angle = 3.0 * PI / 2.0 + position_angle_offset;
            let horizontal = self.half_straight_dist + self.turn_radius * position_angle.cos();
            let vertical = self.turn_radius * position_angle.sin();
            let rot = Quat::from_rotation_z(position_angle + PI / 2.0);
            (Vec2::new(horizontal, vertical), rot)
        } else if current_lap_distance <= self.second_straightaway_dist {
            // in second straightaway
            let horizontal =
                self.half_straight_dist - (current_lap_distance - self.first_turn_dist);
            let vertical = self.vertical_offset;
            let rot = Quat::from_rotation_z(PI);
            (Vec2::new(horizontal, vertical), rot)
        } else if current_lap_distance <= self.second_turn_dist {
            // in second turn
            let circle_dist = current_lap_distance - self.second_straightaway_dist;
            let position_angle_offset = circle_dist / self.turn_radius;
            let position_angle = PI / 2.0 + position_angle_offset;
            let horizontal = -self.half_straight_dist + self.turn_radius * position_angle.cos();
            let vertical = self.turn_radius * position_angle.sin();
            let rot = Quat::from_rotation_z(position_angle + PI / 2.0);
            (Vec2::new(horizontal, vertical), rot)
        } else {
            // in first straightaway again (before finish line)
            let horizontal =
                -self.half_straight_dist + (current_lap_distance - self.second_turn_dist);
            let vertical = -self.vertical_offset;
            let rot = Quat::from_rotation_z(0.0);
            (Vec2::new(horizontal, vertical), rot)
        }
    }

    pub fn finished(&self, distance: f32) -> bool {
        distance >= self.lap_distance * LAPS as f32
    }
}

fn setup(mut commands: Commands, track_texture: Res<TrackTexture>) {
    commands.spawn(SpriteBundle {
        texture: track_texture.default.clone(),
        ..default()
    });
}
