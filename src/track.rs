use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

const LAPS: i32 = 4;
const STRAIGHT_DISTANCE: f32 = 2000.0;
const TURN_RADIUS: f32 = 620.0;
const LANE_WIDTH: f32 = 100.0;

pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrackLanes>();
    }
}

#[derive(Resource, Copy, Clone, Debug)]
pub struct TrackLanes {
    first: TrackLane,
    second: TrackLane,
    third: TrackLane,
    fourth: TrackLane,
}

impl TrackLanes {
    pub fn track_lane(&self, id: &TrackLaneId) -> &TrackLane {
        match id {
            TrackLaneId::First => &self.first,
            TrackLaneId::Second => &self.second,
            TrackLaneId::Third => &self.third,
            TrackLaneId::Fourth => &self.fourth,
        }
    }
}

impl Default for TrackLanes {
    fn default() -> Self {
        Self {
            first: TrackLane::new(&TrackLaneId::First),
            second: TrackLane::new(&TrackLaneId::Second),
            third: TrackLane::new(&TrackLaneId::Third),
            fourth: TrackLane::new(&TrackLaneId::Fourth),
        }
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
    fn length_from_inner_edge(&self) -> f32 {
        let factor = match self {
            TrackLaneId::First => 0,
            TrackLaneId::Second => 1,
            TrackLaneId::Third => 2,
            TrackLaneId::Fourth => 3,
        };
        (LANE_WIDTH / 2.0) + (LANE_WIDTH * factor as f32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackSection {
    FirstStraightawayAfterFinishLine,
    FirstTurn,
    SecondStraightaway,
    SecondTurn,
    FirstStraightawayBeforeFinishLine,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TrackLane {
    length_from_inner_edge: f32,
    lap_distance: f32,
    half_straight_dist: f32,
    turn_radius: f32,
    vertical_offset: f32,
    first_straightaway_after_finish_line_dist: f32,
    first_turn_dist: f32,
    second_straightaway_dist: f32,
    second_turn_dist: f32,
}

impl TrackLane {
    pub fn new(lane: &TrackLaneId) -> Self {
        let length_from_inner_edge = lane.length_from_inner_edge();
        let semicircle_circumfrence = PI * (TURN_RADIUS + length_from_inner_edge);
        let lap_distance = (STRAIGHT_DISTANCE + semicircle_circumfrence) * 2.0;
        let half_straight_dist = STRAIGHT_DISTANCE / 2.0;
        let turn_radius = TURN_RADIUS + length_from_inner_edge;
        let vertical_offset = turn_radius;
        let first_straightaway_after_finish_line_dist = half_straight_dist;
        let first_turn_dist = first_straightaway_after_finish_line_dist + semicircle_circumfrence;
        let second_straightaway_dist = first_turn_dist + STRAIGHT_DISTANCE;
        let second_turn_dist = second_straightaway_dist + semicircle_circumfrence;
        TrackLane {
            length_from_inner_edge,
            lap_distance,
            half_straight_dist,
            turn_radius,
            vertical_offset,
            first_straightaway_after_finish_line_dist,
            first_turn_dist,
            second_straightaway_dist,
            second_turn_dist,
        }
    }

    pub fn current_lap_distance(&self, distance: f32) -> f32 {
        distance % self.lap_distance
    }

    pub fn in_track_section(&self, distance: f32) -> TrackSection {
        let current_lap_distance = self.current_lap_distance(distance);
        if current_lap_distance < self.first_straightaway_after_finish_line_dist {
            TrackSection::FirstStraightawayAfterFinishLine
        } else if current_lap_distance <= self.first_turn_dist {
            TrackSection::FirstTurn
        } else if current_lap_distance <= self.second_straightaway_dist {
            TrackSection::SecondStraightaway
        } else if current_lap_distance <= self.second_turn_dist {
            TrackSection::SecondTurn
        } else {
            TrackSection::FirstStraightawayBeforeFinishLine
        }
    }

    pub fn in_turn(&self, distance: f32) -> bool {
        matches!(
            self.in_track_section(distance),
            TrackSection::FirstTurn | TrackSection::SecondTurn
        )
    }

    pub fn track_section_end_distance(&self, track_section: &TrackSection) -> f32 {
        match track_section {
            TrackSection::FirstStraightawayAfterFinishLine => {
                self.first_straightaway_after_finish_line_dist
            }
            TrackSection::FirstTurn => self.first_turn_dist,
            TrackSection::SecondStraightaway => self.second_straightaway_dist,
            TrackSection::SecondTurn => self.second_turn_dist,
            TrackSection::FirstStraightawayBeforeFinishLine => self.lap_distance,
        }
    }

    pub fn track_section_start_distance(&self, track_section: &TrackSection) -> f32 {
        match track_section {
            TrackSection::FirstStraightawayAfterFinishLine => 0.0,
            TrackSection::FirstTurn => self.first_straightaway_after_finish_line_dist,
            TrackSection::SecondStraightaway => self.first_turn_dist,
            TrackSection::SecondTurn => self.second_straightaway_dist,
            TrackSection::FirstStraightawayBeforeFinishLine => self.second_turn_dist,
        }
    }

    pub fn distance_to_end_of_track_section(&self, distance: f32) -> f32 {
        let current_lap_distance = self.current_lap_distance(distance);
        let current_section = self.in_track_section(current_lap_distance);
        let current_section_end_distance = self.track_section_end_distance(&current_section);
        let distance_to_end = current_section_end_distance - current_lap_distance;
        if distance_to_end.abs() < 0.005 {
            0.0
        } else {
            distance_to_end
        }
    }

    /// Designed to be used for building an arc path
    /// Returns a tuple of (center: Vec2, radii: Vec2, sweep_angle: f32, x_rotation: f32)
    pub fn turn_curve_components(
        &self,
        start_distance: f32,
        end_distance: f32,
    ) -> (Vec2, Vec2, f32, f32) {
        let section = self.in_track_section(start_distance);
        let dist_from_section_start = start_distance - self.track_section_start_distance(&section);
        let radius = TURN_RADIUS + self.length_from_inner_edge;
        let radii = Vec2::new(radius, radius);
        let x_rotation_offset = dist_from_section_start / TURN_RADIUS;
        let sweep_angle = (end_distance - start_distance) / TURN_RADIUS;
        let (center, x_rotation) = if matches!(section, TrackSection::FirstTurn) {
            let center = Vec2::new(self.half_straight_dist, 0.0);
            let x_rotation = -PI / 2.0 + x_rotation_offset;
            (center, x_rotation)
        } else {
            let center = Vec2::new(-self.half_straight_dist, 0.0);
            let x_rotation = PI / 2.0 + x_rotation_offset;
            (center, x_rotation)
        };
        (center, radii, sweep_angle, x_rotation)
    }

    /// Determine the position and rotation at a specified distance
    /// from the starting position of 0.0.
    pub fn position_and_rotation(&self, distance: f32) -> (Vec2, Quat) {
        let current_lap_distance = self.current_lap_distance(distance);
        match self.in_track_section(distance) {
            TrackSection::FirstStraightawayAfterFinishLine => {
                let horizontal = current_lap_distance;
                let vertical = -self.vertical_offset;
                let rot = Quat::from_rotation_z(0.0);
                (Vec2::new(horizontal, vertical), rot)
            }
            TrackSection::FirstTurn => {
                let circle_dist =
                    current_lap_distance - self.first_straightaway_after_finish_line_dist;
                let position_angle_offset = circle_dist / self.turn_radius;
                let position_angle = 3.0 * PI / 2.0 + position_angle_offset;
                let horizontal = self.half_straight_dist + self.turn_radius * position_angle.cos();
                let vertical = self.turn_radius * position_angle.sin();
                let rot = Quat::from_rotation_z(position_angle + FRAC_PI_2);
                (Vec2::new(horizontal, vertical), rot)
            }
            TrackSection::SecondStraightaway => {
                let horizontal =
                    self.half_straight_dist - (current_lap_distance - self.first_turn_dist);
                let vertical = self.vertical_offset;
                let rot = Quat::from_rotation_z(PI);
                (Vec2::new(horizontal, vertical), rot)
            }
            TrackSection::SecondTurn => {
                let circle_dist = current_lap_distance - self.second_straightaway_dist;
                let position_angle_offset = circle_dist / self.turn_radius;
                let position_angle = PI / 2.0 + position_angle_offset;
                let horizontal = -self.half_straight_dist + self.turn_radius * position_angle.cos();
                let vertical = self.turn_radius * position_angle.sin();
                let rot = Quat::from_rotation_z(position_angle + PI / 2.0);
                (Vec2::new(horizontal, vertical), rot)
            }
            TrackSection::FirstStraightawayBeforeFinishLine => {
                let horizontal =
                    -self.half_straight_dist + (current_lap_distance - self.second_turn_dist);
                let vertical = -self.vertical_offset;
                let rot = Quat::from_rotation_z(0.0);
                (Vec2::new(horizontal, vertical), rot)
            }
        }
    }

    pub fn finished(&self, distance: f32) -> bool {
        distance >= self.lap_distance * LAPS as f32
    }
}
