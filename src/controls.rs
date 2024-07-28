mod buttons;
mod mouse;

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::{
    actions::BikeAction,
    bike::Bike,
    loading::IconTextures,
    player::Player,
    track::{TrackLane, TrackLanes},
    RacingState,
};

use self::{
    buttons::{make_button, ActionButton, ActionButtonsPlugin, ButtonRowPositions},
    mouse::MousePlugin,
};

const BIKE_TO_BUTTON_SPACING: f32 = 150.0;
const BUTTON_SPACING: f32 = 80.0;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ActionButtonsPlugin)
            .add_plugins(MousePlugin)
            .add_systems(OnEnter(RacingState::Commanding), on_enter_commanding_state)
            .add_systems(OnEnter(RacingState::Simulating), on_enter_simulating_state);
    }
}

fn on_enter_commanding_state(
    mut commands: Commands,
    q_player_bike: Query<&Bike, With<Player>>,
    icon_textures: Res<IconTextures>,
    track_lanes: Res<TrackLanes>,
) {
    for bike in q_player_bike.iter() {
        let bike_distance = bike.distance;
        let track_lane = track_lanes.track_lane(&bike.current_lane_id);
        let row_0 = button_row_positions(bike_distance, track_lane, 0);
        commands.spawn(make_button(
            BikeAction::LeftHip,
            row_0.left,
            row_0.rotation,
            &icon_textures,
            BikeAction::LeftHip.can_do(bike),
        ));
        commands.spawn(make_button(
            BikeAction::Stop,
            row_0.middle,
            row_0.rotation,
            &icon_textures,
            BikeAction::Stop.can_do(bike),
        ));
        commands.spawn(make_button(
            BikeAction::RightHip,
            row_0.right,
            row_0.rotation,
            &icon_textures,
            BikeAction::RightHip.can_do(bike),
        ));

        let row_1 = button_row_positions(bike_distance, track_lane, 1);
        commands.spawn(make_button(
            BikeAction::LeftElbow,
            row_1.left,
            row_1.rotation,
            &icon_textures,
            BikeAction::LeftElbow.can_do(bike),
        ));
        commands.spawn(make_button(
            BikeAction::Skid,
            row_1.middle,
            row_1.rotation,
            &icon_textures,
            BikeAction::Skid.can_do(bike),
        ));
        commands.spawn(make_button(
            BikeAction::RightElbow,
            row_1.right,
            row_1.rotation,
            &icon_textures,
            BikeAction::RightElbow.can_do(bike),
        ));

        let row_2 = button_row_positions(bike_distance, track_lane, 2);
        commands.spawn(make_button(
            BikeAction::LeftLeft,
            row_2.left,
            row_2.rotation,
            &icon_textures,
            BikeAction::LeftLeft.can_do(bike),
        ));
        commands.spawn(make_button(
            BikeAction::Watch,
            row_2.middle,
            row_2.rotation,
            &icon_textures,
            BikeAction::Watch.can_do(bike),
        ));
        commands.spawn(make_button(
            BikeAction::RightRight,
            row_2.right,
            row_2.rotation,
            &icon_textures,
            BikeAction::RightRight.can_do(bike),
        ));

        let row_3 = button_row_positions(bike_distance, track_lane, 3);
        commands.spawn(make_button(
            BikeAction::Left,
            row_3.left,
            row_3.rotation,
            &icon_textures,
            BikeAction::Left.can_do(bike),
        ));
        commands.spawn(make_button(
            BikeAction::Accelerate,
            row_3.middle,
            row_3.rotation,
            &icon_textures,
            BikeAction::Accelerate.can_do(bike),
        ));
        commands.spawn(make_button(
            BikeAction::Right,
            row_3.right,
            row_3.rotation,
            &icon_textures,
            BikeAction::Right.can_do(bike),
        ));
    }
}

fn on_enter_simulating_state(mut commands: Commands, q_buttons: Query<Entity, With<ActionButton>>) {
    for entity in q_buttons.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_row_positions(
    bike_distance: f32,
    track_lane: &TrackLane,
    row_index: usize,
) -> ButtonRowPositions {
    let distance = bike_distance + BIKE_TO_BUTTON_SPACING + row_index as f32 * BUTTON_SPACING;
    let (position, rotation) = track_lane.position_and_rotation(distance);
    let middle = position.extend(10.0);
    let constant_button_rotation = Quat::from_rotation_z(-FRAC_PI_2);
    let button_rotation = constant_button_rotation.mul_quat(rotation);
    let offset = Vec3::new(0.0, BUTTON_SPACING, 0.0);
    let rotated_offset = rotation.mul_vec3(offset);
    ButtonRowPositions {
        left: middle + rotated_offset,
        middle,
        right: middle - rotated_offset,
        rotation: button_rotation,
    }
}
