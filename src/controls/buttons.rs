use bevy::prelude::*;

use crate::{loading::IconTextures, RacingState};

use super::{
    actions::{ActionEvent, ActionKind},
    mouse::MouseWorldCoords,
};

const MOUSE_TO_BUTTON_DIST: f32 = 1000.0;
const BUTTON_SIZE: f32 = 60.0;

pub struct ActionButtonsPlugin;

impl Plugin for ActionButtonsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (detect_mouse_over_buttons, on_mouse_clicked).run_if(in_state(RacingState::Commanding)),
        )
        .add_systems(
            Update,
            (on_mouse_over_added, on_mouse_over_removed).after(detect_mouse_over_buttons),
        );
    }
}

fn detect_mouse_over_buttons(
    mut commands: Commands,
    mouse_world_coords: Res<MouseWorldCoords>,
    q_buttons: Query<(Entity, &Transform, Option<&MouseOver>), With<ActionButton>>,
) {
    for (entity, transform, maybe_mouse_over) in q_buttons.iter() {
        let dist_to_button = (mouse_world_coords.0 - transform.translation.xy()).length_squared();
        if dist_to_button < MOUSE_TO_BUTTON_DIST {
            if maybe_mouse_over.is_none() {
                commands.entity(entity).insert(MouseOver);
            }
        } else if maybe_mouse_over.is_some() {
            commands.entity(entity).remove::<MouseOver>();
        }
    }
}

fn on_mouse_over_added(mut commands: Commands, q_buttons: Query<Entity, Added<MouseOver>>) {
    for entity in q_buttons.iter() {
        commands
            .spawn((
                MouseOverHighlight,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, 0.2),
                        custom_size: Some(Vec2::splat(BUTTON_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, -1.0),
                    ..default()
                },
            ))
            .set_parent(entity);
    }
}

fn on_mouse_over_removed(
    mut commands: Commands,
    mut removed_mouse_over: RemovedComponents<MouseOver>,
    q_highlights: Query<(Entity, &Parent), With<MouseOverHighlight>>,
) {
    for button_entity in removed_mouse_over.read() {
        for (highlight_entity, parent) in q_highlights.iter() {
            if parent.get() == button_entity {
                commands
                    .entity(button_entity)
                    .remove_children(&[highlight_entity]);
                commands.entity(highlight_entity).despawn();
            }
        }
    }
}

fn on_mouse_clicked(
    mut action_event: EventWriter<ActionEvent>,
    q_buttons: Query<&ActionKind, With<MouseOver>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_released(MouseButton::Left) {
        for action_kind in &q_buttons {
            action_event.send(ActionEvent(*action_kind));
        }
    }
}

#[derive(Bundle)]
pub struct ActionButtonBundle {
    pub action_button: ActionButton,
    pub kind: ActionKind,
    pub sprite: SpriteBundle,
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ActionButton;

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug)]
pub struct MouseOver;

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug)]
pub struct MouseOverHighlight;

pub struct ButtonRowPositions {
    pub left: Vec3,
    pub middle: Vec3,
    pub right: Vec3,
    pub rotation: Quat,
}

pub fn make_button(
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
        action_button: ActionButton,
        kind: action_kind,
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
