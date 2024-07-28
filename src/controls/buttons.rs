use bevy::prelude::*;

use crate::{
    actions::{ActionEvent, BikeAction},
    loading::IconTextures,
    player::Player,
    RacingState,
};

use super::mouse::MouseWorldCoords;

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
    q_buttons: Query<(Entity, &Transform, &ActionButton, Option<&MouseOver>)>,
) {
    for (entity, transform, action_button, maybe_mouse_over) in q_buttons.iter() {
        if action_button.enabled {
            let dist_to_button =
                (mouse_world_coords.0 - transform.translation.xy()).length_squared();
            if dist_to_button < MOUSE_TO_BUTTON_DIST {
                if maybe_mouse_over.is_none() {
                    commands.entity(entity).insert(MouseOver);
                }
            } else if maybe_mouse_over.is_some() {
                commands.entity(entity).remove::<MouseOver>();
            }
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
    q_buttons: Query<&BikeAction, With<MouseOver>>,
    buttons: Res<ButtonInput<MouseButton>>,
    q_player: Query<Entity, With<Player>>,
) {
    if buttons.just_released(MouseButton::Left) {
        for action_kind in &q_buttons {
            for player in &q_player {
                action_event.send(ActionEvent::new(player, *action_kind));
            }
        }
    }
}

#[derive(Bundle)]
pub struct ActionButtonBundle {
    pub action_button: ActionButton,
    pub kind: BikeAction,
    pub sprite: SpriteBundle,
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ActionButton {
    enabled: bool,
}

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
    action_kind: BikeAction,
    position: Vec3,
    rotation: Quat,
    icon_textures: &IconTextures,
    enabled: bool,
) -> ActionButtonBundle {
    let texture = match action_kind {
        BikeAction::Accelerate => icon_textures.accelerate.clone(),
        BikeAction::Watch => icon_textures.watch.clone(),
        BikeAction::Skid => icon_textures.skid.clone(),
        BikeAction::Stop => icon_textures.stop.clone(),
        BikeAction::Left => icon_textures.left.clone(),
        BikeAction::LeftLeft => icon_textures.left_left.clone(),
        BikeAction::LeftElbow => icon_textures.left_elbow.clone(),
        BikeAction::LeftHip => icon_textures.left_hip.clone(),
        BikeAction::Right => icon_textures.right.clone(),
        BikeAction::RightRight => icon_textures.right_right.clone(),
        BikeAction::RightElbow => icon_textures.right_elbow.clone(),
        BikeAction::RightHip => icon_textures.right_hip.clone(),
    };
    let sprite_alpha = if enabled { 1.0 } else { 0.3 };
    ActionButtonBundle {
        action_button: ActionButton { enabled },
        kind: action_kind,
        sprite: SpriteBundle {
            transform: Transform {
                translation: position,
                rotation,
                scale: Vec3::splat(1.0),
            },
            texture,
            sprite: Sprite {
                color: Color::Srgba(Srgba::new(1.0, 1.0, 1.0, sprite_alpha)),
                ..default()
            },
            ..default()
        },
    }
}
