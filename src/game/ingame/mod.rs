use bevy::app::App;
use bevy::prelude::{in_state, IntoSystemConfigs, OnEnter, Plugin, PostUpdate, PreUpdate, Update};

use crate::game::ingame::action_points::reset_action_points;
use crate::game::ingame::combat::{CombatEvent, CombatPlugin};
use crate::game::ingame::egui::ui_system;
use crate::game::ingame::game_log::{display_log_events, handle_log_events, LogEvent, LogRecord};
use crate::game::ingame::health_bar::{
    add_health_bars, update_health_bar_positions, update_health_bar_size, HealthBarResources,
};
use crate::game::ingame::hovered_hex::{update_hovered_hex, HoveredHex, HoveredUnitResource};
use crate::game::ingame::input_system::{handle_selected_unit_input, update_hovered_unit};
use crate::game::ingame::move_unit::MoveUnitsPlugin;
use crate::game::ingame::path::{compute_current_path, despawn_old_path, CurrentPath};
use crate::game::ingame::post_update_systems::update_transform_from_hex;
use crate::game::ingame::selected_unit::{
    check_whether_selected_unit_needs_recomputation, reset_selected_unit, update_hex_overlay,
    update_reachable_hexes_cache, update_selected_unit_hex, SelectedUnitResource,
};
use crate::game::ingame::unit_status::update_engagement;
use crate::game::menu::menu_ui;
use crate::game::states::game_state::GameState;
use crate::game::states::in_game_state::InGameState;
use crate::game::states::round_state::{round_end_system, ActiveTeam, RoundState};

pub mod action_points;
pub mod combat;
mod egui;
pub mod game_log;
mod health_bar;
pub mod hex;
pub mod hovered_hex;
mod input_system;
mod move_unit;
mod path;
pub mod post_update_systems;
pub mod selected_unit;
pub mod team_setup;
pub mod terrain;
pub mod unit;
pub mod unit_status;
pub mod z_ordering;

pub struct IngameLogicPlugin;

impl Plugin for IngameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MoveUnitsPlugin, CombatPlugin))
            .init_state::<RoundState>()
            .add_event::<LogEvent>()
            .add_event::<CombatEvent>()
            .init_resource::<ActiveTeam>()
            .init_resource::<HoveredHex>()
            .init_resource::<SelectedUnitResource>()
            .init_resource::<HoveredUnitResource>()
            .init_resource::<HealthBarResources>()
            .init_resource::<LogRecord>()
            .init_resource::<CurrentPath>()
            .add_systems(
                PreUpdate,
                update_hovered_hex.run_if(in_state(RoundState::Input)),
            )
            .add_systems(Update, menu_ui.run_if(in_state(GameState::Loading)))
            .add_systems(
                PreUpdate,
                (update_transform_from_hex, update_reachable_hexes_cache)
                    .run_if(in_state(InGameState::Playing)),
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
                    (handle_selected_unit_input, update_hovered_unit)
                        .run_if(in_state(RoundState::Input)),
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
                    update_engagement,
                )
                    .run_if(in_state(InGameState::Playing)),
            )
            .add_systems(
                OnEnter(RoundState::RoundEnd),
                (round_end_system, reset_action_points),
            );
    }
}
