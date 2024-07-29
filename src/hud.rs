use bevy::prelude::*;

use crate::{bike::Bike, game::LapEvent, opponent::Opponent, player::Player, PlayingState};

const HUD_FONT_SIZE: f32 = 20.0;
const HUD_TEXT_PADDING: Val = Val::Px(5.0);
const POSITION_VERTICAL_SPACE: Val = Val::Px(5.0);
const LAP_VERTICAL_SPACE: Val = Val::Px(45.0);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayingState::SetupRace), setup)
            .add_systems(OnExit(PlayingState::Racing), teardown)
            .add_systems(
                Update,
                (update_laps, update_position).run_if(in_state(PlayingState::Racing)),
            );
    }
}

#[derive(Component)]
struct HudElement;

#[derive(Component)]
struct LapDisplay;

#[derive(Component)]
struct PositionDisplay;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    // LAPS
    commands.spawn((
        HudElement,
        LapDisplay,
        TextBundle::from_sections([
            TextSection::new(
                "Laps: ",
                TextStyle {
                    font_size: HUD_FONT_SIZE,
                    color: TEXT_COLOR,
                    font: font_handle.clone(),
                },
            ),
            TextSection::new(
                "0",
                TextStyle {
                    font_size: HUD_FONT_SIZE,
                    color: SCORE_COLOR,
                    font: font_handle.clone(),
                },
            ),
            TextSection::new(
                "/4",
                TextStyle {
                    font_size: HUD_FONT_SIZE,
                    color: TEXT_COLOR,
                    font: font_handle.clone(),
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: LAP_VERTICAL_SPACE,
            left: HUD_TEXT_PADDING,
            ..default()
        }),
    ));

    // Position
    commands.spawn((
        HudElement,
        PositionDisplay,
        TextBundle::from_sections([
            TextSection::new(
                "Position: ",
                TextStyle {
                    font_size: HUD_FONT_SIZE,
                    color: TEXT_COLOR,
                    font: font_handle.clone(),
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: HUD_FONT_SIZE,
                color: SCORE_COLOR,
                font: font_handle.clone(),
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: POSITION_VERTICAL_SPACE,
            left: HUD_TEXT_PADDING,
            ..default()
        }),
    ));
}

fn teardown(mut commands: Commands, q_hud: Query<Entity, With<HudElement>>) {
    for entity in &q_hud {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_laps(
    mut lap_event: EventReader<LapEvent>,
    mut q_lap_display: Query<&mut Text, With<LapDisplay>>,
) {
    for event in lap_event.read() {
        let mut text = q_lap_display.single_mut();
        text.sections[1].value = event.0.to_string();
    }
}

fn update_position(
    q_opponents: Query<&Bike, With<Opponent>>,
    q_player: Query<&Bike, With<Player>>,
    mut q_position_display: Query<&mut Text, With<PositionDisplay>>,
) {
    if let Ok(player_bike) = q_player.get_single() {
        let player_distance = player_bike.distance;
        let mut opponent_distances = Vec::new();
        for opponent_bike in &q_opponents {
            opponent_distances.push(opponent_bike.distance);
        }
        let mut player_pos = 4;
        for opponent_distance in opponent_distances {
            if player_distance > opponent_distance {
                player_pos -= 1;
            }
        }
        let mut text = q_position_display.single_mut();
        text.sections[1].value = player_pos.to_string();
    }
}
