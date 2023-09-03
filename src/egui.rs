use bevy::prelude::{warn, NextState, Query, Res, ResMut};
use bevy_egui::egui::{Ui, Window};
use bevy_egui::EguiContexts;

use crate::action_points::ActionPoints;
use crate::combat::HealthPoints;
use crate::game_state::{ActiveTeam, RoundState};
use crate::hex::HexComponent;
use crate::hovered_hex::{HoveredHex, HoveredUnitResource};
use crate::selected_unit::SelectedUnitResource;
use crate::team_setup::Team;
use crate::terrain::Terrain;

pub fn ui_system(
    mut contexts: EguiContexts,
    active_team: Res<ActiveTeam>,
    mut round_state: ResMut<NextState<RoundState>>,
    selected_unit_resource: Res<SelectedUnitResource>,
    hovered_unit_resource: Res<HoveredUnitResource>,
    units: Query<(&ActionPoints, &HealthPoints, &Team)>,
    hovered_hex: Res<HoveredHex>,
    terrain_hexes: Query<(&Terrain, &HexComponent)>,
) {
    Window::new("Round").show(contexts.ctx_mut(), |ui| {
        ui.heading(format!("Round of {}", active_team.0));

        ui.separator();

        display_selected_unit(selected_unit_resource, hovered_unit_resource, units, ui);

        ui.separator();

        display_terrain(hovered_hex, terrain_hexes, ui);

        ui.separator();

        if ui.button("End Round").clicked() {
            round_state.set(RoundState::RoundEnd);
        };
    });
}

fn display_selected_unit(
    selected_unit_resource: Res<SelectedUnitResource>,
    hovered_unit_resource: Res<HoveredUnitResource>,
    units: Query<(&ActionPoints, &HealthPoints, &Team)>,
    ui: &mut Ui,
) {
    ui.heading("Unit:");

    let unit_to_show = hovered_unit_resource
        .0
        .or(selected_unit_resource.selected_unit());

    let Some(selected_unit) = unit_to_show else {
        ui.label("-");
        return;
    };

    let (action_points, health_points, team) = units.get(selected_unit).unwrap();
    ui.label(format!("Owner: {team}"));
    ui.label(format!(
        "Action points: {}/{}",
        action_points.left,
        action_points.get_max()
    ));
    ui.label(format!(
        "Health points: {}/{}",
        health_points.left,
        health_points.get_max()
    ));
}

fn display_terrain(
    hovered_hex: Res<HoveredHex>,
    terrain_hexes: Query<(&Terrain, &HexComponent)>,
    ui: &mut Ui,
) {
    ui.heading("Terrain:");
    let Some(hovered_hex) = hovered_hex.0 else {
        ui.label("-");
        return;
    };

    let Some((terrain, _)) = terrain_hexes
        .iter()
        .find(|(_, hex_component)| hex_component.0 == hovered_hex)
    else {
        warn!("Did not find terrain for hex {hovered_hex:?}");
        ui.label("-");
        return;
    };

    ui.label(&terrain.name);
    ui.label(format!("Movement cost: {}", terrain.movement_cost));
}
