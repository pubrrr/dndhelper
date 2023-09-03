use bevy::log::{Level, LogPlugin};
use bevy::prelude::{
    default, in_state, App, Camera2dBundle, Commands, IntoSystemConfigs, OnEnter, PluginGroup,
    PostUpdate, PreUpdate, Startup, Update, WindowPlugin,
};
use bevy::DefaultPlugins;
use bevy_asset_loader::prelude::LoadingStateAppExt;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_egui::EguiPlugin;

use dndhelper::action_points::reset_action_points;
use dndhelper::combat::{despawn_dead_units, handle_combat, CombatantsResource};
use dndhelper::egui::ui_system;
use dndhelper::game_state::start_round_system;
use dndhelper::game_state::{round_end_system, ActiveTeam, GameState, RoundState};
use dndhelper::health_bar::{
    add_health_bars, update_health_bar_positions, update_health_bar_size, HealthBarResources,
};
use dndhelper::hex::setup_hex_grid;
use dndhelper::hovered_hex::HoveredUnitResource;
use dndhelper::hovered_hex::{update_hovered_hex, HoveredHex};
use dndhelper::input_system::{handle_selected_unit_input, update_hovered_unit};
use dndhelper::menu::menu_ui;
use dndhelper::nation_assets::{DynamicNationAssets, LoadingState, NationAssetCollection};
use dndhelper::post_update_systems::update_transform_from_hex;
#[cfg(not(target_family = "wasm"))]
use dndhelper::scan_assets::write_nations_assets;
use dndhelper::scan_assets::GENERATED_NATIONS_ASSETS_FILE;
use dndhelper::selected_unit::{
    check_whether_selected_unit_needs_recomputation, reset_selected_unit, update_hex_overlay,
    update_reachable_hexes_cache, update_selected_unit_hex, SelectedUnitResource,
};
use dndhelper::team_setup::setup_team_units;

fn main() {
    #[cfg(not(target_family = "wasm"))]
    write_nations_assets().unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(bevy::window::Window {
                        resolution: (1000., 1000.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::DEBUG,
                    filter: "wgpu=error,naga=warn,bevy_render=info,bevy_app=info".to_string(),
                }),
            RonAssetPlugin::<DynamicNationAssets>::new(&["assets.ron"]),
            EguiPlugin,
        ))
        .add_loading_state(
            bevy_asset_loader::loading_state::LoadingState::new(LoadingState::Loading)
                .continue_to_state(LoadingState::Done)
                .set_standard_dynamic_asset_collection_file_endings(vec![]),
        )
        .add_dynamic_collection_to_loading_state::<_, DynamicNationAssets>(
            LoadingState::Loading,
            GENERATED_NATIONS_ASSETS_FILE,
        )
        .add_collection_to_loading_state::<_, NationAssetCollection>(LoadingState::Loading)
        .add_state::<LoadingState>()
        .add_state::<GameState>()
        .add_state::<RoundState>()
        .add_systems(Startup, setup_camera)
        .add_systems(
            OnEnter(GameState::InGame),
            (setup_hex_grid, setup_team_units, start_round_system),
        )
        .add_systems(
            PreUpdate,
            update_hovered_hex.run_if(in_state(RoundState::Moving)),
        )
        .add_systems(Update, menu_ui.run_if(in_state(GameState::Loading)))
        .add_systems(
            Update,
            (
                ui_system,
                add_health_bars,
                reset_selected_unit,
                handle_selected_unit_input.run_if(in_state(RoundState::Moving)),
                update_hovered_unit.run_if(in_state(RoundState::Moving)),
                handle_combat.run_if(in_state(RoundState::Combat)),
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            PostUpdate,
            (
                check_whether_selected_unit_needs_recomputation,
                update_transform_from_hex,
                update_selected_unit_hex,
                update_reachable_hexes_cache,
                update_hex_overlay,
                despawn_dead_units,
                update_health_bar_positions,
                update_health_bar_size,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            OnEnter(RoundState::RoundEnd),
            (round_end_system, reset_action_points),
        )
        .init_resource::<ActiveTeam>()
        .init_resource::<HoveredHex>()
        .init_resource::<SelectedUnitResource>()
        .init_resource::<HoveredUnitResource>()
        .init_resource::<CombatantsResource>()
        .init_resource::<HealthBarResources>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
