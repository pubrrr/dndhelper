use bevy::log::{Level, LogPlugin};
use bevy::prelude::{
    default, in_state, App, Camera2dBundle, Commands, IntoSystemConfigs, OnEnter, PluginGroup,
    PostUpdate, PreUpdate, Startup, Update, WindowPlugin,
};
use bevy::DefaultPlugins;
use bevy_asset_loader::prelude::LoadingStateAppExt;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_egui::EguiPlugin;

use dndhelper::common::DynamicNationAssetsDefinition;
use dndhelper::game::action_points::reset_action_points;
use dndhelper::game::combat::{despawn_dead_units, handle_combat, CombatantsResource};
use dndhelper::game::egui::ui_system;
use dndhelper::game::game_log::{display_log_events, handle_log_events, LogEvent, LogRecord};
use dndhelper::game::game_state::start_round_system;
use dndhelper::game::game_state::{round_end_system, ActiveTeam, GameState, RoundState};
use dndhelper::game::health_bar::{
    add_health_bars, update_health_bar_positions, update_health_bar_size, HealthBarResources,
};
use dndhelper::game::hex::setup_hex_grid;
use dndhelper::game::hovered_hex::HoveredUnitResource;
use dndhelper::game::hovered_hex::{update_hovered_hex, HoveredHex};
use dndhelper::game::input_system::{handle_selected_unit_input, update_hovered_unit};
use dndhelper::game::menu::menu_ui;
use dndhelper::game::nation_asset_resource::insert_nation_assets_resource;
use dndhelper::game::nation_asset_resource::NationAssetsResourceHelperAssets;
use dndhelper::game::nation_assets::{LoadingState, NationAssetCollection, UnitStats};
use dndhelper::game::post_update_systems::update_transform_from_hex;
use dndhelper::game::selected_unit::{
    check_whether_selected_unit_needs_recomputation, reset_selected_unit, update_hex_overlay,
    update_reachable_hexes_cache, update_selected_unit_hex, SelectedUnitResource,
};
use dndhelper::game::team_setup::setup_team_units;
#[cfg(not(target_family = "wasm"))]
use dndhelper::scan_assets::write_nations_assets;
use dndhelper::scan_assets::GENERATED_NATIONS_ASSETS_FILE;

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
            RonAssetPlugin::<DynamicNationAssetsDefinition>::new(&["assets.ron"]),
            RonAssetPlugin::<UnitStats>::new(&["stats.ron"]),
            EguiPlugin,
        ))
        .add_loading_state(
            bevy_asset_loader::loading_state::LoadingState::new(LoadingState::LoadingDynamicAssets)
                .continue_to_state(LoadingState::LoadingNationAssetsDefinition)
                .set_standard_dynamic_asset_collection_file_endings(vec![]),
        )
        .add_loading_state(
            bevy_asset_loader::loading_state::LoadingState::new(
                LoadingState::LoadingNationAssetsDefinition,
            )
            .continue_to_state(LoadingState::Done)
            .set_standard_dynamic_asset_collection_file_endings(vec![]),
        )
        .add_dynamic_collection_to_loading_state::<_, DynamicNationAssetsDefinition>(
            LoadingState::LoadingDynamicAssets,
            GENERATED_NATIONS_ASSETS_FILE,
        )
        .add_collection_to_loading_state::<_, NationAssetCollection>(
            LoadingState::LoadingDynamicAssets,
        )
        .add_collection_to_loading_state::<_, NationAssetsResourceHelperAssets>(
            LoadingState::LoadingNationAssetsDefinition,
        )
        .add_state::<LoadingState>()
        .add_state::<GameState>()
        .add_state::<RoundState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(LoadingState::Done), insert_nation_assets_resource)
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
                display_log_events,
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
                handle_log_events,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            OnEnter(RoundState::RoundEnd),
            (round_end_system, reset_action_points),
        )
        .add_event::<LogEvent>()
        .init_resource::<ActiveTeam>()
        .init_resource::<HoveredHex>()
        .init_resource::<SelectedUnitResource>()
        .init_resource::<HoveredUnitResource>()
        .init_resource::<CombatantsResource>()
        .init_resource::<HealthBarResources>()
        .init_resource::<LogRecord>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
