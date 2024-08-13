use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

const STRAIGHT_DISTANCE: f32 = 2000.0;
const TURN_RADIUS: f32 = 620.0;
const LANE_WIDTH: f32 = 100.0;

pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Track>();
    }
}

#[derive(Component)]
pub struct TrackVisual;

#[derive(Component, Copy, Clone, Debug)]
pub struct TrackPosition {
    distance_from_start: f32,
    pub distance_from_inner_edge: f32,
}

impl TrackPosition {
    pub fn new(lane_id: TrackLaneId) -> Self {
        let distance_from_inner_edge = lane_id.distance_from_inner_edge();
        Self {
            distance_from_start: 0.0,
            distance_from_inner_edge,
        }
    }

    pub fn total_distance(&self) -> f32 {
        self.distance_from_start
    }

    pub fn distance_on_track(&self, track: &Track) -> f32 {
        track.distance_from_start_in_bounds(self.distance_from_start)
    }

    pub fn advance(&mut self, distance_movement: f32, track: &Track) {
        let distance_on_track = self.distance_on_track(track);
        let adjusted_distance_movement = track.movement_at_distance_from_inner_edge(
            distance_on_track,
            self.distance_from_inner_edge,
            distance_movement,
        );
        self.distance_from_start += adjusted_distance_movement;
    }

    pub fn set_distance_from_inside(&mut self, distance_from_inside: f32) {
        self.distance_from_inner_edge = distance_from_inside;
    }

    pub fn position_and_rotation(&self, track: &Track) -> (Vec2, Quat) {
        track.position_and_rotation(
            track.distance_from_start_in_bounds(self.distance_from_start),
            self.distance_from_inner_edge,
        )
    }

    pub fn in_turn(&self, track: &Track) -> bool {
        track.in_turn(self.distance_from_start)
    }
}

#[derive(Resource)]
pub struct Track {
    semicircle_circumfrence: f32,
    vertical_offset: f32,
    half_straight_distance: f32,
    first_straightaway_after_finish_line_dist: f32,
    first_turn_dist: f32,
    second_straightaway_dist: f32,
    second_turn_dist: f32,
    total_distance: f32,
}

impl Default for Track {
    fn default() -> Self {
        Self::new()
    }
}

impl Track {
    pub fn new() -> Self {
        let semicircle_circumfrence = PI * TURN_RADIUS;
        let vertical_offset = TURN_RADIUS;
        let half_straight_distance = STRAIGHT_DISTANCE / 2.0;
        let total_distance = (STRAIGHT_DISTANCE + semicircle_circumfrence) * 2.0;
        let first_straightaway_after_finish_line_dist = half_straight_distance;
        let first_turn_dist = first_straightaway_after_finish_line_dist + semicircle_circumfrence;
        let second_straightaway_dist = first_turn_dist + STRAIGHT_DISTANCE;
        let second_turn_dist = second_straightaway_dist + semicircle_circumfrence;
        Self {
            semicircle_circumfrence,
            vertical_offset,
            half_straight_distance,
            first_straightaway_after_finish_line_dist,
            first_turn_dist,
            second_straightaway_dist,
            second_turn_dist,
            total_distance,
        }
    }

    fn in_track_section(&self, distance: f32) -> TrackSection {
        if distance <= self.first_straightaway_after_finish_line_dist {
            TrackSection::FirstStraightawayAfterFinishLine
        } else if distance <= self.first_turn_dist {
            TrackSection::FirstTurn
        } else if distance <= self.second_straightaway_dist {
            TrackSection::SecondStraightaway
        } else if distance <= self.second_turn_dist {
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

    fn movement_at_distance_from_inner_edge(
        &self,
        distance_from_start: f32,
        distance_from_inner_edge: f32,
        movement: f32,
    ) -> f32 {
        match self.in_track_section(distance_from_start) {
            TrackSection::FirstStraightawayAfterFinishLine
            | TrackSection::SecondStraightaway
            | TrackSection::FirstStraightawayBeforeFinishLine => movement,
            TrackSection::FirstTurn | TrackSection::SecondTurn => {
                let turn_distance = PI * (TURN_RADIUS + distance_from_inner_edge);
                let movement_factor = self.semicircle_circumfrence / turn_distance;
                movement * movement_factor
            }
        }
    }

    fn distance_from_start_in_bounds(&self, distance_from_start: f32) -> f32 {
        distance_from_start % self.total_distance
    }

    pub fn num_laps_completed(&self, distance_from_start: f32) -> usize {
        (distance_from_start / self.total_distance).floor() as usize
    }

    /// Determine the position and rotation at a specified distance
    /// from the starting position of 0.0.
    pub fn position_and_rotation(
        &self,
        distance_from_start: f32,
        distance_from_inner_edge: f32,
    ) -> (Vec2, Quat) {
        let turn_radius = TURN_RADIUS + distance_from_inner_edge;
        let distance_on_track = self.distance_from_start_in_bounds(distance_from_start);
        match self.in_track_section(distance_on_track) {
            TrackSection::FirstStraightawayAfterFinishLine => {
                let horizontal = distance_on_track;
                let vertical = -self.vertical_offset - distance_from_inner_edge;
                let rot = Quat::from_rotation_z(0.0);
                (Vec2::new(horizontal, vertical), rot)
            }
            TrackSection::FirstTurn => {
                let circle_dist =
                    distance_on_track - self.first_straightaway_after_finish_line_dist;
                let position_angle_offset = circle_dist / TURN_RADIUS;
                let position_angle = 3.0 * PI / 2.0 + position_angle_offset;
                let horizontal = self.half_straight_distance + turn_radius * position_angle.cos();
                let vertical = turn_radius * position_angle.sin();
                let rot = Quat::from_rotation_z(position_angle + FRAC_PI_2);
                (Vec2::new(horizontal, vertical), rot)
            }
            TrackSection::SecondStraightaway => {
                let horizontal =
                    self.half_straight_distance - (distance_on_track - self.first_turn_dist);
                let vertical = self.vertical_offset + distance_from_inner_edge;
                let rot = Quat::from_rotation_z(PI);
                (Vec2::new(horizontal, vertical), rot)
            }
            TrackSection::SecondTurn => {
                let circle_dist = distance_on_track - self.second_straightaway_dist;
                let position_angle_offset = circle_dist / TURN_RADIUS;
                let position_angle = PI / 2.0 + position_angle_offset;
                let horizontal = -self.half_straight_distance + turn_radius * position_angle.cos();
                let vertical = turn_radius * position_angle.sin();
                let rot = Quat::from_rotation_z(position_angle + PI / 2.0);
                (Vec2::new(horizontal, vertical), rot)
            }
            TrackSection::FirstStraightawayBeforeFinishLine => {
                let horizontal =
                    -self.half_straight_distance + (distance_on_track - self.second_turn_dist);
                let vertical = -self.vertical_offset - distance_from_inner_edge;
                let rot = Quat::from_rotation_z(0.0);
                (Vec2::new(horizontal, vertical), rot)
            }
        }
    }

    /// Designed to be used for building an arc path
    /// Returns a tuple of (center: Vec2, radii: Vec2, sweep_angle: f32, x_rotation: f32)
    pub fn turn_curve_components(
        &self,
        start_distance: f32,
        end_distance: f32,
        distance_from_inner_edge: f32,
    ) -> (Vec2, Vec2, f32, f32) {
        let section = self.in_track_section(start_distance);
        let dist_from_section_start = start_distance - self.track_section_start_distance(&section);
        let radius = TURN_RADIUS + distance_from_inner_edge;
        let radii = Vec2::new(radius, radius);
        let x_rotation_offset = dist_from_section_start / TURN_RADIUS;
        let sweep_angle = (end_distance - start_distance) / TURN_RADIUS;
        let (center, x_rotation) = if matches!(section, TrackSection::FirstTurn) {
            let center = Vec2::new(self.half_straight_distance, 0.0);
            let x_rotation = -PI / 2.0 + x_rotation_offset;
            (center, x_rotation)
        } else {
            let center = Vec2::new(-self.half_straight_distance, 0.0);
            let x_rotation = PI / 2.0 + x_rotation_offset;
            (center, x_rotation)
        };
        (center, radii, sweep_angle, x_rotation)
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

    pub fn track_section_end_distance(&self, track_section: &TrackSection) -> f32 {
        match track_section {
            TrackSection::FirstStraightawayAfterFinishLine => {
                self.first_straightaway_after_finish_line_dist
            }
            TrackSection::FirstTurn => self.first_turn_dist,
            TrackSection::SecondStraightaway => self.second_straightaway_dist,
            TrackSection::SecondTurn => self.second_turn_dist,
            TrackSection::FirstStraightawayBeforeFinishLine => self.total_distance,
        }
    }

    pub fn distance_to_end_of_track_section(&self, distance: f32) -> f32 {
        let current_lap_distance = self.distance_from_start_in_bounds(distance);
        let current_section = self.in_track_section(current_lap_distance);
        let current_section_end_distance = self.track_section_end_distance(&current_section);
        let distance_to_end = current_section_end_distance - current_lap_distance;
        if distance_to_end.abs() < 0.005 {
            0.0
        } else {
            distance_to_end
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
    pub fn distance_from_inner_edge(&self) -> f32 {
        let factor = match self {
            TrackLaneId::First => 0,
            TrackLaneId::Second => 1,
            TrackLaneId::Third => 2,
            TrackLaneId::Fourth => 3,
        };
        (LANE_WIDTH / 2.0) + (LANE_WIDTH * factor as f32)
    }

    pub fn left(&self) -> TrackLaneId {
        match self {
            TrackLaneId::First => TrackLaneId::First,
            TrackLaneId::Second => TrackLaneId::First,
            TrackLaneId::Third => TrackLaneId::Second,
            TrackLaneId::Fourth => TrackLaneId::Third,
        }
    }

    pub fn left_left(&self) -> TrackLaneId {
        self.left().left()
    }

    pub fn right(&self) -> TrackLaneId {
        match self {
            TrackLaneId::First => TrackLaneId::Second,
            TrackLaneId::Second => TrackLaneId::Third,
            TrackLaneId::Third => TrackLaneId::Fourth,
            TrackLaneId::Fourth => TrackLaneId::Fourth,
        }
    }

    pub fn right_right(&self) -> TrackLaneId {
        self.right().right()
    }

    pub fn between(&self, other: TrackLaneId) -> TrackLaneId {
        let self_index = *self as u8;
        let other_index = other as u8;
        if other_index > self_index {
            self.right()
        } else {
            self.left()
        }
    }

    pub fn difference(&self, other: TrackLaneId) -> i32 {
        let self_index = *self as i32;
        let other_index = other as i32;
        self_index - other_index
    }

    pub fn is_to_right_of(&self, other: TrackLaneId) -> bool {
        let self_index = *self as i32;
        let other_index = other as i32;
        self_index - other_index > 0
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
