// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod actions;
mod bike;
mod camera;
mod collision;
mod controls;
mod game;
mod hud;
mod loading;
mod menu;
mod opponent;
mod path_highlight;
mod player;
mod random;
mod track;

use actions::ActionsPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bike::BikePlugin;
use camera::CameraDollyPlugin;
use collision::CollisionPlugin;
use controls::ControlsPlugin;
use game::GamePlugin;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use opponent::OpponentPlugin;
use path_highlight::PathHighlightPlugin;
use player::PlayerPlugin;
use random::RandomnessPlugin;
use track::TrackPlugin;

#[derive(States, Default, PartialEq, Eq, Hash, Clone, Debug)]
enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
}

#[derive(SubStates, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[source(GameState = GameState::Playing)]
enum PlayingState {
    #[default]
    SetupRace,
    Racing,
    FinishRace,
}

#[derive(SubStates, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[source(PlayingState = PlayingState::Racing)]
enum RacingState {
    #[default]
    Commanding,
    Simulating,
}

fn main() {
    //std::env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics in web builds on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
            LoadingPlugin,
            RandomnessPlugin,
            GamePlugin,
            MenuPlugin,
            CameraDollyPlugin,
            TrackPlugin,
            BikePlugin,
            CollisionPlugin,
            ActionsPlugin,
            PlayerPlugin,
            OpponentPlugin,
            ControlsPlugin,
            PathHighlightPlugin,
        ))
        .init_state::<GameState>()
        .add_sub_state::<PlayingState>()
        .add_sub_state::<RacingState>()
        .run();
}
