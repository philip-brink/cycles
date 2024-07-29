use bevy::prelude::*;

use crate::{player::Player, GameState, PlayingState};

const BUTTON_NORMAL_COLOR: Color = Color::srgb(0.15, 0.15, 0.15);
const BUTTON_HOVERED_COLOR: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_PRESSED_COLOR: Color = Color::srgb(0.35, 0.75, 0.35);
const BUTTON_WIDTH: f32 = 150.0;
const BUTTON_HEIGHT: f32 = 65.0;
const BUTTON_FONT_SIZE: f32 = 40.0;
const BUTTON_FONT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

pub struct FinishRacePlugin;

impl Plugin for FinishRacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayingState::FinishRace), setup_position_display)
            .add_systems(OnExit(PlayingState::FinishRace), teardown)
            .add_systems(
                Update,
                button_system.run_if(in_state(PlayingState::FinishRace)),
            );
    }
}

#[derive(Component)]
struct FinishRaceDisplay;

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
enum ButtonAction {
    Menu,
    Quit,
}

fn setup_position_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_player: Query<&Player>,
) {
    if let Ok(player) = q_player.get_single() {
        let position_text = match player.position {
            1 => "WINNER",
            2 => "SECOND",
            3 => "THIRD",
            4 => "LAST PLACE",
            _ => "Wow, terrible!",
        };
        let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
        commands
            .spawn((
                FinishRaceDisplay,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    FinishRaceDisplay,
                    TextBundle::from_sections([TextSection::new(
                        position_text,
                        TextStyle {
                            font_size: 100.0,
                            color: Color::srgba(0.9, 0.1, 0.1, 0.8),
                            font: font_handle.clone(),
                        },
                    )]),
                ));
                parent
                    .spawn((ButtonAction::Menu, make_button()))
                    .with_children(|parent| {
                        parent.spawn(make_button_text("Menu", font_handle.clone()));
                    });
                parent
                    .spawn((ButtonAction::Quit, make_button()))
                    .with_children(|parent| {
                        parent.spawn(make_button_text("Quit", font_handle.clone()));
                    });
            });
    }
}

fn teardown(mut commands: Commands, q_elements: Query<Entity, With<FinishRaceDisplay>>) {
    for entity in &q_elements {
        commands.entity(entity).despawn_recursive();
    }
}

fn make_button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Px(BUTTON_WIDTH),
            height: Val::Px(BUTTON_HEIGHT),
            border: UiRect::all(Val::Px(2.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        border_radius: BorderRadius::MAX,
        background_color: BUTTON_NORMAL_COLOR.into(),
        ..default()
    }
}

fn make_button_text(text: &str, font_handle: Handle<Font>) -> TextBundle {
    TextBundle::from_section(
        text,
        TextStyle {
            font: font_handle,
            font_size: BUTTON_FONT_SIZE,
            color: BUTTON_FONT_COLOR,
        },
    )
}

fn button_system(
    mut interaction_query: Query<
        (&ButtonAction, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for (button_action, interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED_COLOR.into();
                match button_action {
                    ButtonAction::Menu => {
                        game_state.set(GameState::Menu);
                    }
                    ButtonAction::Quit => {
                        app_exit_events.send(AppExit::Success);
                    }
                };
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVERED_COLOR.into();
            }
            Interaction::None => {
                *color = BUTTON_NORMAL_COLOR.into();
            }
        }
    }
}
