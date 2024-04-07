use bevy::app::App;
#[cfg(not(test))]
use bevy::prelude::PreUpdate;
use bevy::prelude::{
    debug, in_state, ButtonInput, Commands, Event, EventReader, EventWriter, IntoSystemConfigs,
    MouseButton, NextState, OnEnter, OnExit, Plugin, PostUpdate, Query, Res, ResMut, Resource,
    Transform, Update, Vec3, With,
};
use hexx::Hex;

use crate::game::asset_loading::nation_asset_resource::NationAssetsResource;
use crate::game::asset_loading::nation_assets::UnitKey;
use crate::game::ingame::action_points::ActionPoints;
use crate::game::ingame::combat::{CombatConfig, HealthPoints};
use crate::game::ingame::hex::{setup_hex_grid, HexComponent, HexMarker};
#[cfg(not(test))]
use crate::game::ingame::hovered_hex::update_hovered_hex;
use crate::game::ingame::hovered_hex::HoveredHex;
#[cfg(not(test))]
use crate::game::ingame::post_update_systems::update_transform_from_hex;
use crate::game::ingame::team_setup::Team;
use crate::game::ingame::terrain::{MovementCost, Terrain};
use crate::game::ingame::unit::{ProtoUnitBundle, UnitBundle, UnitMarker};
use crate::game::ingame::z_ordering::ZOrdering;
#[cfg(not(test))]
use crate::game::states::in_game_state::deploy_units::ui::deploy_units_menu;
use crate::game::states::in_game_state::InGameState;
use crate::game::states::round_state::ActiveTeam;

pub struct DeployUnitsPlugin;

impl Plugin for DeployUnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DeployUnitEvent>()
            .add_event::<DeploymentDoneEvent>()
            .add_systems(
                OnEnter(InGameState::DeployUnits),
                (setup_deploy_units_resources, setup_hex_grid),
            )
            .add_systems(
                OnExit(InGameState::DeployUnits),
                clean_up_deploy_units_resources,
            );

        #[cfg(not(test))]
        app.add_systems(
            PreUpdate,
            (update_transform_from_hex, update_hovered_hex)
                .run_if(in_state(InGameState::DeployUnits)),
        )
        .add_systems(
            Update,
            deploy_units_menu.run_if(in_state(InGameState::DeployUnits)),
        );

        app.add_systems(
            Update,
            deploy_units_input_system.run_if(in_state(InGameState::DeployUnits)),
        )
        .add_systems(
            PostUpdate,
            (handle_deploy_unit_event, handle_deployment_done_event)
                .run_if(in_state(InGameState::DeployUnits)),
        );
    }
}

#[derive(Event)]
pub(super) struct DeploymentDoneEvent;

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
pub(super) struct SelectedUnitToDeploy(pub Option<UnitKey>);

fn setup_deploy_units_resources(mut commands: Commands) {
    commands.init_resource::<DeployPoints>();
    commands.init_resource::<SelectedUnitToDeploy>();
}

fn clean_up_deploy_units_resources(mut commands: Commands) {
    commands.remove_resource::<DeployPoints>();
    commands.remove_resource::<SelectedUnitToDeploy>();
}

#[cfg(not(test))]
mod ui {
    use bevy::prelude::{EventWriter, Res, ResMut};
    use bevy_egui::egui::Window;
    use bevy_egui::EguiContexts;

    use crate::game::asset_loading::nation_asset_resource::NationAssetsResource;
    use crate::game::states::in_game_state::deploy_units::{
        DeployPoints, DeploymentDoneEvent, SelectedUnitToDeploy,
    };
    use crate::game::states::in_game_state::PickedNationsResource;
    use crate::game::states::round_state::ActiveTeam;

    pub(super) fn deploy_units_menu(
        mut contexts: EguiContexts,
        active_player: Res<ActiveTeam>,
        deploy_points: Res<DeployPoints>,
        picked_nations_resource: Res<PickedNationsResource>,
        nation_assets_resource: Res<NationAssetsResource>,
        mut selected_unit_to_deploy: ResMut<SelectedUnitToDeploy>,
        mut deployment_done_event: EventWriter<DeploymentDoneEvent>,
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
                deployment_done_event.send(DeploymentDoneEvent);
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

            let selected_unit_assets = nation_assets_resource.get_unit_assets(selected_unit);
            ui.label(format!("Selected Unit: {:#?}", selected_unit_assets.stats));
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn deploy_units_input_system(
    buttons: Res<ButtonInput<MouseButton>>,
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
    for event in deploy_unit_events.read() {
        let unit_assets = nation_assets_resource.get_unit_assets(&event.unit);

        let unit_bundle: UnitBundle = ProtoUnitBundle {
            texture: unit_assets.image.clone(),
            transform: Transform::from_xyz(0., 0., ZOrdering::UNITS).with_scale(Vec3::splat(0.5)),
            unit_marker: UnitMarker(unit_assets.stats.name.clone()),
            player: event.player,
            action_points: ActionPoints::new(
                unit_assets.stats.max_action_points,
                unit_assets.stats.max_attacks_per_round,
                unit_assets.stats.attack_action_point_cost,
            ),
            health_points: HealthPoints::new(unit_assets.stats.max_health_points),
            combat_config: CombatConfig {
                attack: unit_assets.stats.attack,
                defense: unit_assets.stats.defense,
                range: unit_assets.stats.range,
            },
            hex: event.hex,
        }
        .into();

        let entity = commands.spawn(unit_bundle).id();

        debug!("Deployed unit {entity:?} for event: {event:?}");
    }
}

fn handle_deployment_done_event(
    mut deployment_done_events: EventReader<DeploymentDoneEvent>,
    mut in_game_state: ResMut<NextState<InGameState>>,
    mut active_player: ResMut<ActiveTeam>,
    mut deploy_points: ResMut<DeployPoints>,
    mut selected_unit_to_deploy: ResMut<SelectedUnitToDeploy>,
) {
    if deployment_done_events.read().next().is_some() {
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
}
