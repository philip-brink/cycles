use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    math::bounding::{BoundingCircle, IntersectsVolume},
    prelude::*,
};

use crate::{bike::Bike, loading::IconTextures, RacingState};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEvent>().add_systems(
            Update,
            (
                check_for_bike_collisions,
                remove_collisions,
                on_event_collision,
                update_collision_indicator,
            )
                .run_if(in_state(RacingState::Simulating)),
        );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Collider {
    half_size: Vec2,
}

impl Collider {
    pub fn new(width: f32, length: f32) -> Self {
        Self {
            half_size: Vec2::new(width / 2.0, length / 2.0),
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Collision {
    pub other_entity: Entity,
    pub side: CollisionSide,
    pub other_bike_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionSide {
    Front,
    Left,
    Right,
    Back,
}

#[derive(Event, Debug, Clone, Copy)]
struct CollisionEvent {
    position: Vec2,
    rotation: Quat,
    bike_entity: Entity,
}

#[derive(Component)]
struct CollisionIndicator {
    timer: Timer,
}

fn check_for_bike_collisions(
    q_colliders: Query<(Entity, &Collider, &Bike, &Transform, Option<&Collision>)>,
    mut commands: Commands,
    mut collision_event: EventWriter<CollisionEvent>,
) {
    for (entity, collider, bike, transform, maybe_collision) in &q_colliders {
        for (other_entity, other_collider, other_bike, other_transform, _) in &q_colliders {
            if entity != other_entity {
                let collision_exists =
                    find_collision(transform, collider, other_transform, other_collider);
                if collision_exists && maybe_collision.is_none() {
                    let distance_difference = bike.distance - other_bike.distance;
                    let track_center = Vec2::ZERO;
                    let dist_from_center = track_center.distance(transform.translation.xy());
                    let other_dist_from_center =
                        track_center.distance(other_transform.translation.xy());
                    let lane_difference = dist_from_center - other_dist_from_center;
                    // multiply distance by 2 since the length of the bike is a lot longer than the width
                    let collision_side = if bike.current_lane_id == other_bike.current_lane_id {
                        if distance_difference > 0.0 {
                            CollisionSide::Back
                        } else {
                            CollisionSide::Front
                        }
                    } else if lane_difference > 0.0 {
                        CollisionSide::Left
                    } else {
                        CollisionSide::Right
                    };
                    let (event_position, event_rotation) = match collision_side {
                        CollisionSide::Front => {
                            (Vec2::new(40.0, 0.0), Quat::from_rotation_z(FRAC_PI_2))
                        }
                        CollisionSide::Left => (Vec2::new(0.0, -20.0), Quat::from_rotation_z(0.0)),
                        CollisionSide::Right => (Vec2::new(0.0, 20.0), Quat::from_rotation_z(PI)),
                        CollisionSide::Back => {
                            (Vec2::new(-40.0, 0.0), Quat::from_rotation_z(-FRAC_PI_2))
                        }
                    };
                    collision_event.send(CollisionEvent {
                        position: event_position,
                        rotation: event_rotation,
                        bike_entity: entity,
                    });
                    println!("Add collision with bike {entity} and {other_entity} on side {collision_side:?}");
                    commands.entity(entity).insert(Collision {
                        other_entity,
                        side: collision_side,
                        other_bike_speed: other_bike.speed,
                    });
                }
            }
        }
    }
}

fn remove_collisions(
    q_collisions: Query<(Entity, &Transform, &Collider, &Collision)>,
    q_bike_transform: Query<(&Transform, &Collider), With<Bike>>,
    mut commands: Commands,
) {
    let mut collisions_to_remove = Vec::new();
    for (entity, transform, collider, collision) in q_collisions.iter() {
        if let Ok((other_transform, other_collider)) = q_bike_transform.get(collision.other_entity)
        {
            if !find_collision(transform, collider, other_transform, other_collider) {
                collisions_to_remove.push(entity);
            }
        }
    }
    for entity in collisions_to_remove {
        commands.entity(entity).remove::<Collision>();
    }
}

fn on_event_collision(
    mut event_collision: EventReader<CollisionEvent>,
    icon_textures: Res<IconTextures>,
    mut commands: Commands,
) {
    for event in event_collision.read() {
        commands
            .spawn((
                CollisionIndicator {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                },
                SpriteBundle {
                    transform: Transform {
                        translation: event.position.extend(7.0),
                        rotation: event.rotation,
                        scale: Vec3::splat(0.5),
                    },
                    texture: icon_textures.collision.clone(),
                    ..default()
                },
            ))
            .set_parent(event.bike_entity);
    }
}

fn update_collision_indicator(
    mut q_collision_indicators: Query<(Entity, &mut CollisionIndicator)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut indicator) in q_collision_indicators.iter_mut() {
        if indicator.timer.finished() {
            commands.entity(entity).despawn_recursive();
        } else {
            indicator.timer.tick(time.delta());
        }
    }
}

fn find_collision(
    transform: &Transform,
    collider: &Collider,
    other_transform: &Transform,
    other_collider: &Collider,
) -> bool {
    // First bike colliders
    let front_wheel_transform = transform.translation
        + transform
            .rotation
            .mul_vec3(Vec3::new(collider.half_size.x, 0.0, 0.0));
    let rear_wheel_transform = transform.translation
        - transform
            .rotation
            .mul_vec3(Vec3::new(collider.half_size.x, 0.0, 0.0));
    let front_wheel_circle = BoundingCircle::new(front_wheel_transform.xy(), collider.half_size.y);
    let rear_wheel_circle = BoundingCircle::new(rear_wheel_transform.xy(), collider.half_size.y);

    // Second bike colliders
    let other_front_wheel_transform = other_transform.translation
        + other_transform
            .rotation
            .mul_vec3(Vec3::new(other_collider.half_size.x, 0.0, 0.0));
    let other_rear_wheel_transform = other_transform.translation
        - other_transform
            .rotation
            .mul_vec3(Vec3::new(other_collider.half_size.x, 0.0, 0.0));
    let other_front_wheel_circle =
        BoundingCircle::new(other_front_wheel_transform.xy(), other_collider.half_size.y);
    let other_rear_wheel_circle =
        BoundingCircle::new(other_rear_wheel_transform.xy(), other_collider.half_size.y);

    // Collision if any intersections exist
    front_wheel_circle.intersects(&other_front_wheel_circle)
        || front_wheel_circle.intersects(&other_rear_wheel_circle)
        || rear_wheel_circle.intersects(&other_front_wheel_circle)
        || rear_wheel_circle.intersects(&other_rear_wheel_circle)
}
