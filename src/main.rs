use bevy::asset::AssetServer;
use bevy::prelude::{
    default, in_state, App, Camera2dBundle, Commands, Entity, Handle, Image, IntoSystemConfigs,
    OnEnter, PluginGroup, PostStartup, PostUpdate, PreUpdate, Resource, Startup, Update,
    WindowPlugin,
};
use bevy::DefaultPlugins;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy_egui::EguiPlugin;

use crate::action_points::reset_action_points;
use crate::combat::{despawn_dead_units, handle_combat, CombatantsResource};
use crate::egui::ui_system;
use crate::game_state::{round_end_system, ActiveTeam, GameState, RoundState};
use crate::health_bar::{
    add_health_bars, update_health_bar_positions, update_health_bar_size, HealthBarResources,
};
use crate::hex::setup_hex_grid;
use crate::hovered_hex::{update_hovered_hex, HoveredHex};
use crate::input_system::{handle_selected_unit_input, update_hovered_unit};
use crate::post_update_systems::update_transform_from_hex;
use crate::selected_unit::{
    reset_selected_unit, update_hex_overlay, update_reachable_hexes_cache,
    update_selected_unit_hex, SelectedUnitResource,
};
use crate::team_setup::setup_team_units;

mod action_points;
mod combat;
mod common_components;
mod egui;
mod game_state;
mod health_bar;
mod hex;
mod hovered_hex;
mod input_system;
mod post_update_systems;
mod selected_unit;
mod team_setup;
mod terrain;
mod util;
mod z_ordering;

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
        .add_state::<RoundState>()
        .add_systems(Startup, (setup_camera, setup_hex_grid))
        .add_systems(PostStartup, setup_team_units)
        .add_systems(
            PreUpdate,
            update_hovered_hex.run_if(in_state(RoundState::Moving)),
        )
        .add_systems(
            Update,
            (
                ui_system,
                add_health_bars,
                reset_selected_unit,
                handle_selected_unit_input.run_if(in_state(RoundState::Moving)),
                update_hovered_unit.run_if(in_state(RoundState::Moving)),
                handle_combat.run_if(in_state(RoundState::Combat)),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                update_transform_from_hex,
                update_selected_unit_hex,
                update_reachable_hexes_cache,
                update_hex_overlay,
                despawn_dead_units,
                update_health_bar_positions,
                update_health_bar_size,
            ),
        )
        .add_systems(
            OnEnter(GameState::RoundEnd),
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

#[derive(Resource, Default)]
pub struct HoveredUnitResource(pub Option<Entity>);

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "manf.png")]
    manf: Handle<Image>,
    #[asset(path = "tree2.png")]
    tree: Handle<Image>,
}
