// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod bike;
mod camera;
mod game;
mod loading;
mod menu;
mod player;
mod race;
mod track;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bike::BikePlugin;
use camera::CameraDollyPlugin;
use game::GamePlugin;
use loading::LoadingPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;
use race::RacePlugin;
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
enum RaceState {
    #[default]
    Simulating,
    Commanding,
    Paused,
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
            GamePlugin,
            MenuPlugin,
            CameraDollyPlugin,
            TrackPlugin,
            BikePlugin,
            PlayerPlugin,
            RacePlugin,
        ))
        .init_state::<GameState>()
        .add_sub_state::<PlayingState>()
        .add_sub_state::<RaceState>()
        .run();
}
