use bevy::log::{Level, LogPlugin};
use bevy::prelude::{default, App, Camera2dBundle, Commands, PluginGroup, Startup, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;

use dndhelper::game::asset_loading::AssetLoadingPlugin;
use dndhelper::game::ingame::IngameLogicPlugin;
use dndhelper::game::states::game_state::GameState;
use dndhelper::game::states::in_game_state::StartupFlowPlugin;
use dndhelper::game::states::quickstart::QuickstartPlugin;
#[cfg(not(target_family = "wasm"))]
use dndhelper::scan_assets::write_nations_assets;

fn main() {
    #[cfg(not(target_family = "wasm"))]
    write_nations_assets().unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        resolution: (1600., 1200.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::DEBUG,
                    filter: "wgpu=error,naga=warn,bevy_render=info,bevy_app=info".to_string(),
                    update_subscriber: None,
                }),
            EguiPlugin,
            AssetLoadingPlugin,
            StartupFlowPlugin,
            QuickstartPlugin,
            IngameLogicPlugin,
            ShapePlugin,
        ))
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
