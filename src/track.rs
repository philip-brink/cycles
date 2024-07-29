use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

pub const LAPS: usize = 4;
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

impl TrackLanes {
    pub fn pos_and_rot_between_lanes(
        &self,
        lane_id_1: TrackLaneId,
        lane_id_2: TrackLaneId,
        distance: f32,
        proportion: f32,
    ) -> (Vec2, Quat) {
        let lane_1 = self.track_lane(&lane_id_1);
        let (pos_1, rot_1) = lane_1.position_and_rotation(distance);
        if lane_id_1 == lane_id_2 {
            return (pos_1, rot_1);
        }
        let lane_2 = self.track_lane(&lane_id_2);
        let lane_2_distance = self.distance_on_adjacent_lane(lane_id_1, lane_id_2, distance);
        let (pos_2, rot_2) = lane_2.position_and_rotation(lane_2_distance);
        let pos_lerp = pos_1.lerp(pos_2, proportion);
        let rot_lerp = rot_1.lerp(rot_2, proportion);
        (pos_lerp, rot_lerp)
    }

    // TODO: The problem is that going from one lane's distance to another lane's distance
    // is wildly different, ESPECIALLY as the race goes on and more laps are finished, as
    // it compounds over time. I need to have the distance automatically adjusted so that
    // if I switch from an inner lane to an outer lane, it won't result in the bike slowing
    // down or even going backwards. Likewise when going inwards, the bike will jump forward
    pub fn distance_on_adjacent_lane(
        &self,
        lane_id_1: TrackLaneId,
        lane_id_2: TrackLaneId,
        distance: f32,
    ) -> f32 {
        let lane_1 = self.track_lane(&lane_id_1);
        let num_laps_completed = lane_1.laps_finished(distance);
        let current_lap_distance = lane_1.current_lap_distance(distance);
        let track_section = lane_1.in_track_section(current_lap_distance);
        let track_section_total_distance = lane_1.track_section_total_distance(&track_section);
        let track_section_remaining_distance =
            lane_1.distance_to_end_of_track_section(current_lap_distance);
        let track_section_proportion =
            1.0 - track_section_remaining_distance / track_section_total_distance;
        let lane_2 = self.track_lane(&lane_id_2);
        let lane_2_current_lap_distance = lane_2
            .lap_distance_at_track_section_proportion(&track_section, track_section_proportion);
        (lane_2.lap_distance * num_laps_completed as f32) + lane_2_current_lap_distance
    }
}

#[derive(Component)]
pub struct Track;

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

    pub fn track_section_total_distance(&self, track_section: &TrackSection) -> f32 {
        match track_section {
            TrackSection::FirstStraightawayAfterFinishLine => {
                self.first_straightaway_after_finish_line_dist
            }
            TrackSection::FirstTurn => {
                self.first_turn_dist - self.first_straightaway_after_finish_line_dist
            }
            TrackSection::SecondStraightaway => {
                self.second_straightaway_dist - self.first_turn_dist
            }
            TrackSection::SecondTurn => self.second_turn_dist - self.second_straightaway_dist,
            TrackSection::FirstStraightawayBeforeFinishLine => {
                self.lap_distance - self.second_turn_dist
            }
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

    pub fn lap_distance_at_track_section_proportion(
        &self,
        track_section: &TrackSection,
        proportion: f32,
    ) -> f32 {
        let initial_dist = self.track_section_start_distance(track_section);
        let section_dist = self.track_section_total_distance(track_section) * proportion;
        initial_dist + section_dist
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

    pub fn laps_finished(&self, distance: f32) -> usize {
        (distance / self.lap_distance).floor() as usize
    }
}
