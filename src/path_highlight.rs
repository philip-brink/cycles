use std::f32::consts::{PI, SQRT_2};

use bevy::{
    color::palettes::css::PURPLE,
    math::bounding::{Aabb2d, Bounded2d, BoundingCircle},
    prelude::*,
    render::render_asset::RenderAssetUsages,
    sprite::MaterialMesh2dBundle,
};

use crate::track::TrackLaneId;
use crate::{bike::Bike, player::Player};

pub struct PathHighlightPlugin;

impl Plugin for PathHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShowPathHighlightEvent>()
            .add_event::<HidePathHighlightEvent>()
            .add_systems(Update, (show_path_highlight, hide_path_highlight));
    }
}

#[derive(Event, Debug, Default, Copy, Clone)]
pub struct ShowPathHighlightEvent;

#[derive(Event, Debug, Default, Copy, Clone)]
pub struct HidePathHighlightEvent;

#[derive(Component)]
struct PathHighlight;

fn show_path_highlight(
    mut show_path_event_reader: EventReader<ShowPathHighlightEvent>,
    bikes: Query<&Bike, With<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in show_path_event_reader.read() {
        for bike in bikes.iter() {
            let (pos, _) = bike.position_and_direction();
            let transform = Transform::from_translation(pos.extend(1.0));
            commands.spawn((
                MaterialMesh2dBundle {
                    // mesh: meshes.add(Rectangle::default()).into(),
                    mesh: meshes
                        .add(PathHighlightShape::new(bike.distance, 2000.0, bike.lane_id).mesh())
                        .into(),
                    // transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    transform,
                    material: color_materials.add(Color::from(PURPLE)),
                    ..default()
                },
                PathHighlight,
            ));
        }
    }
}

fn hide_path_highlight(
    mut hide_path_event_reader: EventReader<HidePathHighlightEvent>,
    mut commands: Commands,
    q_path_highlights: Query<Entity, With<PathHighlight>>,
) {
    for _ in hide_path_event_reader.read() {
        for entity in q_path_highlights.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// A custom 2D heart primitive. The heart is made up of two circles centered at `Vec2::new(±radius, 0.)` each with the same `radius`.
/// The tip of the heart connects the two circles at a 45° angle from `Vec3::NEG_Y`.
#[derive(Copy, Clone)]
pub struct PathHighlightShape {
    /// The distance along the track
    distance: f32,
    /// The length along the track from the distance to draw
    length: f32,
    /// The lane of the track
    lane: TrackLaneId,
}

// The `Primitive2d` or `Primitive3d` trait is required by almost all other traits for primitives in bevy.
// Depending on your shape, you should implement either one of them.
impl Primitive2d for PathHighlightShape {}

impl PathHighlightShape {
    pub const fn new(distance: f32, length: f32, lane: TrackLaneId) -> Self {
        Self {
            distance,
            length,
            lane,
        }
    }
}

// The `Measured2d` and `Measured3d` traits are used to compute the perimeter, the area or the volume of a primitive.
// If you implement `Measured2d` for a 2D primitive, `Measured3d` is automatically implemented for `Extrusion<T>`.
impl Measured2d for PathHighlightShape {
    fn perimeter(&self) -> f32 {
        self.distance * (2.5 * PI + 2f32.powf(1.5) + 2.0)
    }

    fn area(&self) -> f32 {
        let circle_area = PI * self.distance * self.distance;
        let triangle_area = self.distance * self.distance * (1.0 + 2f32.sqrt()) / 2.0;
        let cutout = triangle_area - circle_area * 3.0 / 16.0;

        2.0 * circle_area + 4.0 * cutout
    }
}

// The `Bounded2d` or `Bounded3d` traits are used to compute the Axis Aligned Bounding Boxes or bounding circles / spheres for primitives.
impl Bounded2d for PathHighlightShape {
    fn aabb_2d(&self, translation: Vec2, rotation: impl Into<Rot2>) -> Aabb2d {
        let rotation = rotation.into();
        // The center of the circle at the center of the right wing of the heart
        let circle_center = rotation * Vec2::new(self.distance, 0.0);
        // The maximum X and Y positions of the two circles of the wings of the heart.
        let max_circle = circle_center.abs() + Vec2::splat(self.distance);
        // Since the two circles of the heart are mirrored around the origin, the minimum position is the negative of the maximum.
        let min_circle = -max_circle;

        // The position of the tip at the bottom of the heart
        let tip_position = rotation * Vec2::new(0.0, -self.distance * (1. + SQRT_2));

        Aabb2d {
            min: translation + min_circle.min(tip_position),
            max: translation + max_circle.max(tip_position),
        }
    }

    fn bounding_circle(&self, translation: Vec2, rotation: impl Into<Rot2>) -> BoundingCircle {
        // The bounding circle of the heart is not at its origin. This `offset` is the offset between the center of the bounding circle and its translation.
        let offset = self.distance / 2f32.powf(1.5);
        // The center of the bounding circle
        let center = translation + rotation.into() * Vec2::new(0.0, -offset);
        // The radius of the bounding circle
        let radius = self.distance * (1.0 + 2f32.sqrt()) - offset;

        BoundingCircle::new(center, radius)
    }
}

// You can use the `Meshable` trait to create a `MeshBuilder` for the primitive.
impl Meshable for PathHighlightShape {
    // The meshbuilder can be used to create the actual mesh for that primitive.
    type Output = HeartMeshBuilder;

    fn mesh(&self) -> Self::Output {
        Self::Output {
            heart: *self,
            resolution: 50,
        }
    }
}

// You can include any additional information needed for meshing the primitive in the meshbuilder.
pub struct HeartMeshBuilder {
    heart: PathHighlightShape,
    // The resolution determines the amount of vertices used for each wing of the heart
    resolution: usize,
}

impl MeshBuilder for HeartMeshBuilder {
    // This is where you should build the actual mesh.
    fn build(&self) -> Mesh {
        let radius = self.heart.distance;
        // The curved parts of each wing (half) of the heart have an angle of `PI * 1.25` or 225°
        let wing_angle = PI * 1.25;

        // We create buffers for the vertices, their normals and UVs, as well as the indices used to connect the vertices.
        let mut vertices = Vec::with_capacity(2 * self.resolution);
        let mut uvs = Vec::with_capacity(2 * self.resolution);
        let mut indices = Vec::with_capacity(6 * self.resolution - 9);
        // Since the heart is flat, we know all the normals are identical already.
        let normals = vec![[0f32, 0f32, 1f32]; 2 * self.resolution];

        // The point in the middle of the two curved parts of the heart
        vertices.push([0.0; 3]);
        uvs.push([0.5, 0.5]);

        // The left wing of the heart, starting from the point in the middle.
        for i in 1..self.resolution {
            let angle = (i as f32 / self.resolution as f32) * wing_angle;
            let (sin, cos) = angle.sin_cos();
            vertices.push([radius * (cos - 1.0), radius * sin, 0.0]);
            uvs.push([0.5 - (cos - 1.0) / 4., 0.5 - sin / 2.]);
        }

        // The bottom tip of the heart
        vertices.push([0.0, radius * (-1. - SQRT_2), 0.0]);
        uvs.push([0.5, 1.]);

        // The right wing of the heart, starting from the bottom most point and going towards the middle point.
        for i in 0..self.resolution - 1 {
            let angle = (i as f32 / self.resolution as f32) * wing_angle - PI / 4.;
            let (sin, cos) = angle.sin_cos();
            vertices.push([radius * (cos + 1.0), radius * sin, 0.0]);
            uvs.push([0.5 - (cos + 1.0) / 4., 0.5 - sin / 2.]);
        }

        // This is where we build all the triangles from the points created above.
        // Each triangle has one corner on the middle point with the other two being adjacent points on the perimeter of the heart.
        for i in 2..2 * self.resolution as u32 {
            indices.extend_from_slice(&[i - 1, i, 0]);
        }

        // Here, the actual `Mesh` is created. We set the indices, vertices, normals and UVs created above and specify the topology of the mesh.
        Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}
