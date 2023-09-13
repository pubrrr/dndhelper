use bevy::log::{Level, LogPlugin};
use bevy::prelude::{
    default, in_state, App, Camera2dBundle, Commands, IntoSystemConfigs, OnEnter, PluginGroup,
    PostUpdate, PreUpdate, Startup, Update, WindowPlugin,
};
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;

use dndhelper::game::action_points::reset_action_points;
use dndhelper::game::asset_loading::AssetLoadingPlugin;
use dndhelper::game::combat::CombatPlugin;
use dndhelper::game::egui::ui_system;
use dndhelper::game::game_log::{display_log_events, handle_log_events, LogEvent, LogRecord};
use dndhelper::game::health_bar::{
    add_health_bars, update_health_bar_positions, update_health_bar_size, HealthBarResources,
};
use dndhelper::game::hovered_hex::HoveredUnitResource;
use dndhelper::game::hovered_hex::{update_hovered_hex, HoveredHex};
use dndhelper::game::input_system::{handle_selected_unit_input, update_hovered_unit};
use dndhelper::game::menu::menu_ui;
use dndhelper::game::move_unit::handle_move_event;
use dndhelper::game::move_unit::MoveUnitEvent;
use dndhelper::game::path::despawn_old_path;
use dndhelper::game::path::{compute_current_path, CurrentPath};
use dndhelper::game::post_update_systems::update_transform_from_hex;
use dndhelper::game::selected_unit::{
    check_whether_selected_unit_needs_recomputation, reset_selected_unit, update_hex_overlay,
    update_reachable_hexes_cache, update_selected_unit_hex, SelectedUnitResource,
};
use dndhelper::game::states::game_state::GameState;
use dndhelper::game::states::in_game_state::{InGameState, StartupFlowPlugin};
use dndhelper::game::states::quickstart::QuickstartPlugin;
use dndhelper::game::states::round_state::{round_end_system, ActiveTeam, RoundState};
use dndhelper::game::unit_status::disengage_apart_units;
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
                        resolution: (1200., 600.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::DEBUG,
                    filter: "wgpu=error,naga=warn,bevy_render=info,bevy_app=info".to_string(),
                }),
            EguiPlugin,
            AssetLoadingPlugin,
            StartupFlowPlugin,
            QuickstartPlugin,
            CombatPlugin,
            ShapePlugin,
        ))
        .add_state::<GameState>()
        .add_state::<RoundState>()
        .add_systems(Startup, setup_camera)
        .add_systems(
            PreUpdate,
            (update_transform_from_hex, update_hovered_hex).run_if(in_state(RoundState::Moving)),
        )
        .add_systems(Update, menu_ui.run_if(in_state(GameState::Loading)))
        .add_systems(
            PreUpdate,
            update_reachable_hexes_cache.run_if(in_state(InGameState::Playing)),
        )
        .add_systems(
            Update,
            (
                ui_system,
                display_log_events,
                add_health_bars,
                reset_selected_unit,
                compute_current_path,
                despawn_old_path,
                handle_selected_unit_input.run_if(in_state(RoundState::Moving)),
                update_hovered_unit.run_if(in_state(RoundState::Moving)),
            )
                .run_if(in_state(InGameState::Playing)),
        )
        .add_systems(
            PostUpdate,
            (
                check_whether_selected_unit_needs_recomputation,
                update_selected_unit_hex,
                update_hex_overlay,
                update_health_bar_positions,
                update_health_bar_size,
                handle_log_events,
                disengage_apart_units,
                handle_move_event,
            )
                .run_if(in_state(InGameState::Playing)),
        )
        .add_systems(
            OnEnter(RoundState::RoundEnd),
            (round_end_system, reset_action_points),
        )
        .add_event::<LogEvent>()
        .add_event::<MoveUnitEvent>()
        .init_resource::<ActiveTeam>()
        .init_resource::<HoveredHex>()
        .init_resource::<SelectedUnitResource>()
        .init_resource::<HoveredUnitResource>()
        .init_resource::<HealthBarResources>()
        .init_resource::<LogRecord>()
        .init_resource::<CurrentPath>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
