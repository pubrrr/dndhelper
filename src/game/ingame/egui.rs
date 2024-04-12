use bevy::ecs::system::SystemId;
use bevy::prelude::{
    warn, Children, Commands, Entity, Event, EventReader, EventWriter, FromWorld, In, Local,
    NextState, Query, Res, ResMut, With, World,
};
use bevy_egui::egui::{Ui, Window};
use bevy_egui::EguiContexts;

use crate::game::abilities::active_abilities::{ActivatedAbilityMarker, ActiveAbility};
use crate::game::ingame::action_points::ActionPoints;
use crate::game::ingame::combat::{CombatConfig, HealthPoints};
use crate::game::ingame::hex::HexComponent;
use crate::game::ingame::hovered_hex::{HoveredHex, HoveredUnitResource};
use crate::game::ingame::selected_unit::SelectedUnitResource;
use crate::game::ingame::team_setup::Team;
use crate::game::ingame::terrain::Terrain;
use crate::game::ingame::unit::UnitMarker;
use crate::game::ingame::unit_status::UnitStatus;
use crate::game::states::round_state::{ActiveTeam, RoundState};

type UnitQuery<'world, 'state, 'a> = Query<
    'world,
    'state,
    (
        &'a UnitMarker,
        &'a ActionPoints,
        &'a HealthPoints,
        &'a Team,
        &'a UnitStatus,
        &'a CombatConfig,
        &'a Children,
    ),
>;

#[derive(Event, Debug, Clone)]
pub enum UiEvent {
    EndRound,
    ActivateAbility(Entity),
}

#[allow(clippy::too_many_arguments)]
pub(super) fn ui_system(
    mut contexts: EguiContexts,
    active_team: Res<ActiveTeam>,
    mut ui_event: EventWriter<UiEvent>,
    selected_unit_resource: Res<SelectedUnitResource>,
    hovered_unit_resource: Res<HoveredUnitResource>,
    units: UnitQuery,
    active_abilities: Query<(Entity, &ActiveAbility)>,
    hovered_hex: Res<HoveredHex>,
    terrain_hexes: Query<(&Terrain, &HexComponent)>,
) {
    Window::new("Round").show(contexts.ctx_mut(), |ui| {
        ui.heading(format!("Round of {}", active_team.0));

        ui.separator();

        display_selected_unit(
            active_team,
            selected_unit_resource,
            hovered_unit_resource,
            units,
            active_abilities,
            &mut ui_event,
            ui,
        );

        ui.separator();

        display_terrain(hovered_hex, terrain_hexes, ui);

        ui.separator();

        if ui.button("End Round").clicked() {
            ui_event.send(UiEvent::EndRound);
        };
    });
}

fn display_selected_unit(
    active_team: Res<ActiveTeam>,
    selected_unit_resource: Res<SelectedUnitResource>,
    hovered_unit_resource: Res<HoveredUnitResource>,
    units: UnitQuery,
    active_abilities: Query<(Entity, &ActiveAbility)>,
    ui_event: &mut EventWriter<UiEvent>,
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

    let Ok((unit_marker, action_points, health_points, team, unit_status, combat_config, children)) =
        units.get(selected_unit)
    else {
        ui.label("-");
        return;
    };

    ui.label(format!("Unit: {}", unit_marker.0));
    ui.label(format!("Owner: {team}"));
    ui.label(format!(
        "Action points: {}/{}",
        action_points.left,
        action_points.get_max()
    ));
    ui.label(format!(
        "Attacks this turn: {}/{}",
        action_points.attacks_this_round,
        action_points.get_max_attacks()
    ));
    ui.label(format!(
        "Health points: {}/{}",
        health_points.left,
        health_points.get_max()
    ));
    ui.label(format!("Status: {unit_status:#?}"));
    ui.label(format!("Combat Stats: {combat_config:#?}"));

    children
        .into_iter()
        .filter_map(|child| active_abilities.get(*child).ok())
        .for_each(|(ability_entity, active_ability)| {
            let belongs_to_active_team = &(*active_team).0 == team;
            let is_enabled = belongs_to_active_team;

            let ability_button = ui.add_enabled(
                is_enabled,
                bevy_egui::egui::Button::new(active_ability.get_display_name()),
            );
            if ability_button.clicked() {
                ui_event.send(UiEvent::ActivateAbility(ability_entity));
            }
        });
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

    let Some((terrain, hex_component)) = terrain_hexes
        .iter()
        .find(|(_, hex_component)| hex_component.0 == hovered_hex)
    else {
        warn!("Did not find terrain for hex {hovered_hex:?}");
        ui.label("-");
        return;
    };

    ui.label(format!(
        "Coordinate: ({},{})",
        hex_component.0.x, hex_component.0.y
    ));
    ui.label(&terrain.name);
    ui.label(format!("Movement cost: {}", terrain.movement_cost));
}

pub(super) struct ActivateAbilityCallback(SystemId<Entity>);

impl FromWorld for ActivateAbilityCallback {
    fn from_world(world: &mut World) -> Self {
        let system_id = world.register_system(handle_activate_ability_event);

        Self(system_id)
    }
}

pub(super) fn handle_ui_event(
    mut events: EventReader<UiEvent>,
    mut commands: Commands,
    mut round_state: ResMut<NextState<RoundState>>,
    activate_ability_callback: Local<ActivateAbilityCallback>,
) {
    for event in events.read() {
        match event {
            UiEvent::EndRound => round_state.set(RoundState::RoundEnd),
            UiEvent::ActivateAbility(ability_entity) => {
                commands.run_system_with_input(activate_ability_callback.0, *ability_entity)
            }
        }
    }
}

fn handle_activate_ability_event(
    ability_entity: In<Entity>,
    mut commands: Commands,
    active_abilities: Query<Entity, With<ActivatedAbilityMarker>>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    let Some(mut entity_commands) = commands.get_entity(*ability_entity) else {
        warn!("Could not find entity for ability {:?}", *ability_entity);
        return;
    };
    entity_commands.insert(ActivatedAbilityMarker);

    for entity in active_abilities.iter() {
        commands.entity(entity).remove::<ActivatedAbilityMarker>();
    }

    round_state.set(RoundState::ActivateAbility);
}
