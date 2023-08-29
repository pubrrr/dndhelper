use bevy::asset::AssetServer;
use bevy::prelude::{
    default, in_state, App, Camera2dBundle, Commands, Entity, Handle, Image, IntoSystemConfigs,
    OnEnter, PluginGroup, PostStartup, PostUpdate, PreUpdate, Resource, Startup, Update,
    WindowPlugin,
};
use bevy::DefaultPlugins;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy_egui::EguiPlugin;

use crate::clicked_hex::{update_clicked_hex, ClickedHex};
use crate::egui::ui_system;
use crate::game_state::{round_end_system, ActiveTeam, GameState};
use crate::hex::setup_hex_grid;
use crate::input_system::handle_input;
use crate::post_update_systems::{update_hex_colors, update_transform_from_hex};
use crate::team_setup::{setup_team_resources, setup_team_units};

mod clicked_hex;
mod common_components;
mod egui;
mod game_state;
mod hex;
mod input_system;
mod post_update_systems;
mod team_setup;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(bevy::window::Window {
                    resolution: (1000., 1000.).into(),
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin,
        ))
        .init_collection::<ImageAssets>()
        .add_state::<GameState>()
        .add_systems(
            Startup,
            (setup_camera, setup_hex_grid, setup_team_resources),
        )
        .add_systems(PostStartup, setup_team_units)
        .add_systems(
            PreUpdate,
            update_clicked_hex.run_if(in_state(GameState::Round)),
        )
        .add_systems(
            Update,
            (ui_system, handle_input.run_if(in_state(GameState::Round))),
        )
        .add_systems(PostUpdate, (update_transform_from_hex, update_hex_colors))
        .add_systems(OnEnter(GameState::RoundEnd), round_end_system)
        .init_resource::<ActiveTeam>()
        .init_resource::<ClickedHex>()
        .init_resource::<SelectedUnitResource>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource, Default)]
pub struct SelectedUnitResource {
    selected_unit: Option<Entity>,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "manf.png")]
    manf: Handle<Image>,
    #[asset(path = "tree2.png")]
    tree: Handle<Image>,
}
