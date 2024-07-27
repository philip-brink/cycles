use bevy::prelude::*;

const CAMERA_MOVEMENT_SPEED: f32 = 600.0;

pub struct CameraDollyPlugin;

impl Plugin for CameraDollyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, move_camera);
    }
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera2dBundle {
            projection: OrthographicProjection {
                near: -1000.0,
                far: 1000.0,
                scale: 2.0,
                ..default()
            },
            ..default()
        },
    ));
}

fn move_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_camera: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let horizontal_movement = if keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        -1.0
    } else if keyboard_input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        1.0
    } else {
        0.0
    };
    let vertical_movement = if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        1.0
    } else if keyboard_input.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        -1.0
    } else {
        0.0
    };
    let camera_movement = Vec3::new(horizontal_movement, vertical_movement, 0.0)
        * time.delta_seconds()
        * CAMERA_MOVEMENT_SPEED;

    for mut transform in q_camera.iter_mut() {
        transform.translation += camera_movement;
    }
}
