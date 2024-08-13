use bevy::{color::palettes::css::PURPLE, prelude::*};
use bevy_prototype_lyon::{
    draw::Stroke, entity::ShapeBundle, path::PathBuilder, plugin::ShapePlugin,
};

use crate::{
    bike::Bike,
    player::Player,
    track::{Track, TrackPosition},
    RacingState,
};

pub struct PathHighlightPlugin;

impl Plugin for PathHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            .add_systems(OnEnter(RacingState::Commanding), show_path_highlight)
            .add_systems(OnEnter(RacingState::Simulating), hide_path_highlight);
    }
}

#[derive(Component)]
struct PathHighlight;

fn show_path_highlight(
    bikes: Query<(&Bike, &TrackPosition), With<Player>>,
    mut commands: Commands,
    track: Res<Track>,
) {
    for (bike, track_position) in bikes.iter() {
        let (pos, _) = track_position.position_and_rotation(&track);
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(pos);
        let mut current_path_length = 0.0;
        let path_length = bike.speed;
        while current_path_length < path_length {
            let path_length_remaining = path_length - current_path_length;
            let path_marker = track_position.distance_on_track(&track) + current_path_length;
            let section_end_distance = track.distance_to_end_of_track_section(path_marker);
            let path_section_end_distance = section_end_distance.min(path_length_remaining);
            let end_distance_along_track = path_marker + path_section_end_distance;
            if track.in_turn(path_marker) {
                let (center, radii, sweep_angle, x_rotation) = track.turn_curve_components(
                    path_marker,
                    end_distance_along_track,
                    track_position.distance_from_inner_edge,
                );
                path_builder.arc(center, radii, sweep_angle, x_rotation);
            } else {
                // draw straightaway
                let (end_pos, _) = track.position_and_rotation(
                    end_distance_along_track,
                    track_position.distance_from_inner_edge,
                );
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
            Stroke::new(PURPLE.with_alpha(0.8), 30.0),
        ));
    }
}

fn hide_path_highlight(
    mut commands: Commands,
    q_path_highlights: Query<Entity, With<PathHighlight>>,
) {
    for entity in q_path_highlights.iter() {
        commands.entity(entity).despawn();
    }
}
