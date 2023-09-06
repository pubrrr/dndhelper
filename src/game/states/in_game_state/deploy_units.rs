use bevy::app::App;
use bevy::prelude::{
    debug, default, in_state, Commands, Event, EventReader, EventWriter, Input, IntoSystemConfigs,
    MouseButton, NextState, OnEnter, OnExit, Plugin, PostUpdate, Query, Res, ResMut, Resource,
    SpriteBundle, Transform, Update, Vec3, With,
};
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;
use hexx::Hex;

use crate::game::action_points::ActionPoints;
use crate::game::combat::{CombatConfig, HealthPoints};
use crate::game::common_components::UnitMarker;
use crate::game::hex::{HexComponent, HexMarker};
use crate::game::hovered_hex::HoveredHex;
use crate::game::nation_asset_resource::NationAssetsResource;
use crate::game::nation_assets::UnitKey;
use crate::game::states::in_game_state::{InGameState, PickedNationsResource};
use crate::game::states::round_state::ActiveTeam;
use crate::game::team_setup::Team;
use crate::game::terrain::{MovementCost, Terrain};
use crate::game::unit_status::UnitStatus;
use crate::game::z_ordering::ZOrdering;

pub struct DeployUnitsPlugin;

impl Plugin for DeployUnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DeployUnitEvent>()
            .add_systems(
                OnEnter(InGameState::DeployUnits),
                setup_deploy_units_resources,
            )
            .add_systems(
                OnExit(InGameState::DeployUnits),
                clean_up_deploy_units_resources,
            )
            .add_systems(
                Update,
                (deploy_units_menu, deploy_units_input_system)
                    .run_if(in_state(InGameState::DeployUnits)),
            )
            .add_systems(
                PostUpdate,
                handle_deploy_unit_event.run_if(in_state(InGameState::DeployUnits)),
            );
    }
}

#[derive(Event, Debug)]
struct DeployUnitEvent {
    player: Team,
    unit: UnitKey,
    hex: Hex,
}

#[derive(Resource, Debug)]
struct DeployPoints(usize);

impl Default for DeployPoints {
    fn default() -> Self {
        DeployPoints(5)
    }
}

#[derive(Resource, Debug, Default)]
struct SelectedUnitToDeploy(Option<UnitKey>);

fn setup_deploy_units_resources(mut commands: Commands) {
    commands.init_resource::<DeployPoints>();
    commands.init_resource::<SelectedUnitToDeploy>();
}

fn clean_up_deploy_units_resources(mut commands: Commands) {
    commands.remove_resource::<DeployPoints>();
    commands.remove_resource::<SelectedUnitToDeploy>();
}

fn deploy_units_menu(
    mut contexts: EguiContexts,
    mut active_player: ResMut<ActiveTeam>,
    mut deploy_points: ResMut<DeployPoints>,
    mut in_game_state: ResMut<NextState<InGameState>>,
    picked_nations_resource: Res<PickedNationsResource>,
    nation_assets_resource: Res<NationAssetsResource>,
    mut selected_unit_to_deploy: ResMut<SelectedUnitToDeploy>,
) {
    Window::new("Deploy Units").show(contexts.ctx_mut(), |ui| {
        let picked_nation = &picked_nations_resource.nations_by_player[&active_player.0];
        let nation = nation_assets_resource.get_nation(&picked_nation.nation);

        ui.heading(format!("Player {} - {}", active_player.0, nation.name));
        ui.label(format!(
            "Deploy points: {}/{}",
            deploy_points.0,
            DeployPoints::default().0
        ));

        if ui.button("Done").clicked() {
            match active_player.0 {
                Team::Red => {
                    selected_unit_to_deploy.0 = None;
                    deploy_points.0 = DeployPoints::default().0;
                    active_player.0 = Team::Blue;
                }
                Team::Blue => {
                    in_game_state.set(InGameState::Playing);
                }
            };
        }

        if deploy_points.0 == 0 {
            return;
        }

        ui.separator();

        let unit_keys = nation_assets_resource.get_units(&picked_nation.nation);

        for unit_key in unit_keys {
            let unit_assets = nation_assets_resource.get_unit_assets(&unit_key);
            if ui.button(unit_assets.stats.name).clicked() {
                selected_unit_to_deploy.0 = Some(unit_key.clone());
            }
        }

        let Some(selected_unit) = &selected_unit_to_deploy.0 else {
            ui.label("Choose a unit...");
            return;
        };
        ui.separator();

        let selected_unit_assets = nation_assets_resource.get_unit_assets(&selected_unit);
        ui.label(format!("Selected Unit: {:#?}", selected_unit_assets.stats));
    });
}

fn deploy_units_input_system(
    buttons: Res<Input<MouseButton>>,
    active_team: Res<ActiveTeam>,
    hovered_hex: Res<HoveredHex>,
    selected_unit_to_deploy: Res<SelectedUnitToDeploy>,
    mut deploy_points: ResMut<DeployPoints>,
    mut deploy_unit_event: EventWriter<DeployUnitEvent>,
    already_deployed_units: Query<&HexComponent, With<UnitMarker>>,
    hexes: Query<(&HexComponent, &Terrain), With<HexMarker>>,
) {
    let Some(selected_unit) = &selected_unit_to_deploy.0 else {
        return;
    };

    if deploy_points.0 == 0 {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(hovered_hex) = &hovered_hex.0 else {
        return;
    };

    let is_already_occupied = already_deployed_units
        .iter()
        .any(|hex_component| &hex_component.0 == hovered_hex);
    if is_already_occupied {
        return;
    }

    let is_impassable = hexes
        .iter()
        .find(|(hex_component, _)| &hex_component.0 == hovered_hex)
        .map(|(_, terrain)| match terrain.movement_cost {
            MovementCost::Impassable => true,
            MovementCost::Passable(_) => false,
        })
        .unwrap_or(true);
    if is_impassable {
        return;
    }

    deploy_unit_event.send(DeployUnitEvent {
        player: active_team.0,
        unit: selected_unit.clone(),
        hex: *hovered_hex,
    });
    deploy_points.0 -= 1;
}

fn handle_deploy_unit_event(
    mut commands: Commands,
    nation_assets_resource: Res<NationAssetsResource>,
    mut deploy_unit_events: EventReader<DeployUnitEvent>,
) {
    for event in deploy_unit_events.iter() {
        let unit_assets = nation_assets_resource.get_unit_assets(&event.unit);

        let entity = commands
            .spawn(SpriteBundle {
                texture: unit_assets.image.clone(),
                transform: Transform::from_xyz(0., 0., ZOrdering::UNITS)
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(event.player)
            .insert(UnitMarker(unit_assets.stats.name.clone()))
            .insert(UnitStatus::new())
            .insert(ActionPoints::new(
                unit_assets.stats.max_action_points,
                unit_assets.stats.max_attacks_per_round,
                unit_assets.stats.attack_action_point_cost,
            ))
            .insert(HealthPoints::new(unit_assets.stats.max_health_points))
            .insert(CombatConfig {
                attack: unit_assets.stats.attack,
                defense: unit_assets.stats.defense,
            })
            .insert(HexComponent(event.hex))
            .id();

        debug!("Deployed unit {entity:?} for event: {event:?}");
    }
}
