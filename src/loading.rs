use bevy::prelude::*;

use crate::GameState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Loading),
            (load_track_texture, load_bike_textures, load_icon_textures).before(finish_loading),
        )
        .add_systems(OnEnter(GameState::Loading), finish_loading);
    }
}

#[derive(Resource)]
pub struct TrackTexture {
    pub default: Handle<Image>,
}

impl FromWorld for TrackTexture {
    fn from_world(world: &mut World) -> Self {
        Self {
            default: world.load_asset("images/track/track.png"),
        }
    }
}

#[derive(Resource)]
pub struct BikeTextures {
    pub straight: Handle<Image>,
    pub turn: Handle<Image>,
}

impl FromWorld for BikeTextures {
    fn from_world(world: &mut World) -> Self {
        Self {
            straight: world.load_asset("images/bike/bike_straight.png"),
            turn: world.load_asset("images/bike/bike_turn.png"),
        }
    }
}

#[derive(Resource)]
pub struct IconTextures {
    pub accelerate: Handle<Image>,
    pub watch: Handle<Image>,
    pub skid: Handle<Image>,
    pub stop: Handle<Image>,
    pub left: Handle<Image>,
    pub left_left: Handle<Image>,
    pub left_elbow: Handle<Image>,
    pub left_hip: Handle<Image>,
    pub right: Handle<Image>,
    pub right_right: Handle<Image>,
    pub right_elbow: Handle<Image>,
    pub right_hip: Handle<Image>,
}

impl FromWorld for IconTextures {
    fn from_world(world: &mut World) -> Self {
        Self {
            accelerate: world.load_asset("images/icons/accelerate.png"),
            watch: world.load_asset("images/icons/watch.png"),
            skid: world.load_asset("images/icons/skid.png"),
            stop: world.load_asset("images/icons/stop.png"),
            left: world.load_asset("images/icons/left.png"),
            left_left: world.load_asset("images/icons/left_left.png"),
            left_elbow: world.load_asset("images/icons/left_elbow.png"),
            left_hip: world.load_asset("images/icons/left_hip.png"),
            right: world.load_asset("images/icons/right.png"),
            right_right: world.load_asset("images/icons/right_right.png"),
            right_elbow: world.load_asset("images/icons/right_elbow.png"),
            right_hip: world.load_asset("images/icons/right_hip.png"),
        }
    }
}

fn load_track_texture(mut commands: Commands) {
    commands.init_resource::<TrackTexture>();
}

fn load_bike_textures(mut commands: Commands) {
    commands.init_resource::<BikeTextures>();
}

fn load_icon_textures(mut commands: Commands) {
    commands.init_resource::<IconTextures>();
}

fn finish_loading(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::Menu);
}
