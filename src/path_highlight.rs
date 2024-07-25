use bevy::{
    color::palettes::css::{BLACK, PURPLE, RED, WHITE},
    prelude::*,
};
use bevy_prototype_lyon::{
    draw::Stroke, entity::ShapeBundle, path::PathBuilder, plugin::ShapePlugin,
};

use crate::{bike::Bike, player::Player};

const PATH_LENGTH: f32 = 750.0;

pub struct PathHighlightPlugin;

impl Plugin for PathHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            .add_event::<ShowPathHighlightEvent>()
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
) {
    for _ in show_path_event_reader.read() {
        for bike in bikes.iter() {
            let lane = bike.lane;
            let (pos, _) = lane.position_and_rotation(bike.distance);
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(pos);
            let mut current_path_length = 0.0;
            while current_path_length < PATH_LENGTH {
                let path_length_remaining = PATH_LENGTH - current_path_length;
                let path_marker = bike.distance + current_path_length;
                let section_end_distance = lane.distance_to_end_of_track_section(path_marker);
                let path_section_end_distance = section_end_distance.min(path_length_remaining);
                let end_distance_along_track = path_marker + path_section_end_distance;
                if lane.in_turn(path_marker) {
                    // draw turn
                    // let start_dist = bike.distance + path_section_end_distance;
                    // let end_dist = path_marker + path_section_end_distance;
                    println!("Start dist: {path_marker}, end dist: {end_distance_along_track}");
                    let (center, radii, sweep_angle, x_rotation) =
                        lane.turn_curve_components(path_marker, end_distance_along_track);
                    println!("Sweep angle: {sweep_angle}, x_rotation: {x_rotation}");
                    path_builder.arc(center, radii, sweep_angle, x_rotation);
                } else {
                    // draw straightaway
                    let (end_pos, _) = lane.position_and_rotation(end_distance_along_track);
                    path_builder.line_to(end_pos);
                }
                // need to add just a little extra to avoid floating point equality problems
                current_path_length += path_section_end_distance + 0.0005;
            }
            let path = path_builder.build();
            commands.spawn((
                PathHighlight,
                ShapeBundle {
                    path,
                    spatial: SpatialBundle {
                        transform: Transform::from_xyz(0., 0., 1.),
                        ..default()
                    },
                    ..default()
                },
                Stroke::new(PURPLE.with_alpha(0.8), 10.0),
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
