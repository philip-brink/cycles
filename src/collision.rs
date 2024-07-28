use bevy::{
    math::bounding::{BoundingCircle, IntersectsVolume},
    prelude::*,
};

use crate::{bike::Bike, RacingState};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (check_for_bike_collisions).run_if(in_state(RacingState::Simulating)),
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

impl CollisionSide {
    fn opposite(&self) -> CollisionSide {
        match self {
            CollisionSide::Front => CollisionSide::Back,
            CollisionSide::Left => CollisionSide::Right,
            CollisionSide::Right => CollisionSide::Left,
            CollisionSide::Back => CollisionSide::Front,
        }
    }
}

fn check_for_bike_collisions(
    q_colliders: Query<(Entity, &Collider, &Bike, &Transform, Option<&Collision>)>,
    mut commands: Commands,
) {
    for (entity, collider, bike, transform, maybe_collision) in &q_colliders {
        for (other_entity, other_collider, other_bike, other_transform, _) in &q_colliders {
            if entity != other_entity {
                let collision_exists =
                    collision(transform, collider, other_transform, other_collider);
                if collision_exists && maybe_collision.is_none() {
                    let distance_difference = bike.distance - other_bike.distance;
                    let track_center = Vec2::ZERO;
                    let dist_from_center = track_center.distance(transform.translation.xy());
                    let other_dist_from_center =
                        track_center.distance(other_transform.translation.xy());
                    let lane_difference = dist_from_center - other_dist_from_center;
                    // multiply distance by 2 since the length of the bike is a lot longer than the width
                    let collision_side = if distance_difference.abs() * 2.0 > lane_difference.abs()
                    {
                        if distance_difference > 0.0 {
                            CollisionSide::Back
                        } else {
                            CollisionSide::Front
                        }
                    } else if lane_difference > 0.0 {
                        CollisionSide::Right
                    } else {
                        CollisionSide::Left
                    };
                    commands.entity(entity).insert(Collision {
                        other_entity,
                        side: collision_side,
                        other_bike_speed: other_bike.speed,
                    });
                } else if !collision_exists && maybe_collision.is_some() {
                    commands.entity(entity).remove::<Collision>();
                }
            }
        }
    }
}

fn collision(
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
