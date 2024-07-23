use bevy::prelude::*;

use crate::GameState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Loading),
            (load_track_texture, load_bike_textures, finish_loading).chain(),
        );
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
        BikeTextures {
            straight: world.load_asset("images/bike/bike_straight.png"),
            turn: world.load_asset("images/bike/bike_turn.png"),
        }
    }
}

fn load_track_texture(mut commands: Commands) {
    commands.init_resource::<TrackTexture>();
}

fn load_bike_textures(mut commands: Commands) {
    commands.init_resource::<BikeTextures>();
}

fn finish_loading(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::Menu);
}
