mod actions;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::{bike::Bike, loading::IconTextures, player::Player, RacingState};

use self::actions::ActionKind;

const BIKE_TO_BUTTON_SPACING: f32 = 150.0;
const BUTTON_SPACING: f32 = 80.0;

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(RacingState::Commanding), on_enter_commanding_state)
            .add_systems(OnEnter(RacingState::Simulating), on_enter_simulating_state);
    }
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug)]
struct ControlButton;

struct ButtonRowPositions {
    left: Vec3,
    middle: Vec3,
    right: Vec3,
    rotation: Quat,
}

#[derive(Bundle)]
struct ActionButtonBundle {
    control_button: ControlButton,
    sprite: SpriteBundle,
}

fn on_enter_commanding_state(
    mut commands: Commands,
    q_player_bike: Query<&Bike, With<Player>>,
    icon_textures: Res<IconTextures>,
) {
    for bike in q_player_bike.iter() {
        let row_0 = button_row_positions(bike, 0);
        commands.spawn(make_button(
            ActionKind::LeftHip,
            row_0.left,
            row_0.rotation,
            &icon_textures,
        ));
        commands.spawn(make_button(
            ActionKind::Stop,
            row_0.middle,
            row_0.rotation,
            &icon_textures,
        ));
        commands.spawn(make_button(
            ActionKind::RightHip,
            row_0.right,
            row_0.rotation,
            &icon_textures,
        ));

        let row_1 = button_row_positions(bike, 1);
        commands.spawn(make_button(
            ActionKind::LeftElbow,
            row_1.left,
            row_1.rotation,
            &icon_textures,
        ));
        commands.spawn(make_button(
            ActionKind::Skid,
            row_1.middle,
            row_1.rotation,
            &icon_textures,
        ));
        commands.spawn(make_button(
            ActionKind::RightElbow,
            row_1.right,
            row_1.rotation,
            &icon_textures,
        ));

        let row_2 = button_row_positions(bike, 2);
        commands.spawn(make_button(
            ActionKind::LeftLeft,
            row_2.left,
            row_2.rotation,
            &icon_textures,
        ));
        commands.spawn(make_button(
            ActionKind::Watch,
            row_2.middle,
            row_2.rotation,
            &icon_textures,
        ));
        commands.spawn(make_button(
            ActionKind::RightRight,
            row_2.right,
            row_2.rotation,
            &icon_textures,
        ));

        let row_3 = button_row_positions(bike, 3);
        commands.spawn(make_button(
            ActionKind::Left,
            row_3.left,
            row_3.rotation,
            &icon_textures,
        ));
        commands.spawn(make_button(
            ActionKind::Accelerate,
            row_3.middle,
            row_3.rotation,
            &icon_textures,
        ));
        commands.spawn(make_button(
            ActionKind::Right,
            row_3.right,
            row_3.rotation,
            &icon_textures,
        ));
    }
}

fn on_enter_simulating_state(
    mut commands: Commands,
    q_buttons: Query<Entity, With<ControlButton>>,
) {
    for entity in q_buttons.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_row_positions(bike: &Bike, row_index: usize) -> ButtonRowPositions {
    let distance = bike.distance + BIKE_TO_BUTTON_SPACING + row_index as f32 * BUTTON_SPACING;
    let (position, rotation) = bike.lane.position_and_rotation(distance);
    let middle = position.extend(3.0);
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

fn make_button(
    action_kind: ActionKind,
    position: Vec3,
    rotation: Quat,
    icon_textures: &IconTextures,
) -> ActionButtonBundle {
    let texture = match action_kind {
        ActionKind::Accelerate => icon_textures.accelerate.clone(),
        ActionKind::Watch => icon_textures.watch.clone(),
        ActionKind::Skid => icon_textures.skid.clone(),
        ActionKind::Stop => icon_textures.stop.clone(),
        ActionKind::Left => icon_textures.left.clone(),
        ActionKind::LeftLeft => icon_textures.left_left.clone(),
        ActionKind::LeftElbow => icon_textures.left_elbow.clone(),
        ActionKind::LeftHip => icon_textures.left_hip.clone(),
        ActionKind::Right => icon_textures.right.clone(),
        ActionKind::RightRight => icon_textures.right_right.clone(),
        ActionKind::RightElbow => icon_textures.right_elbow.clone(),
        ActionKind::RightHip => icon_textures.right_hip.clone(),
    };
    ActionButtonBundle {
        control_button: ControlButton,
        sprite: SpriteBundle {
            transform: Transform {
                translation: position,
                rotation,
                scale: Vec3::splat(1.0),
            },
            texture,
            ..default()
        },
    }
}
