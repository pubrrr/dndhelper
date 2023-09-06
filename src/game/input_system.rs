use bevy::prelude::{
    debug, info, warn, Entity, EventWriter, Input, MouseButton, Query, Res, ResMut, With,
};
use hexx::algorithms::a_star;
use hexx::Hex;
use std::collections::HashSet;

use crate::game::action_points::ActionPoints;
use crate::game::combat::{CombatConfig, CombatEvent};
use crate::game::common_components::UnitFilter;
use crate::game::hex::{HexComponent, HexMarker};
use crate::game::hovered_hex::HoveredHex;
use crate::game::hovered_hex::HoveredUnitResource;
use crate::game::selected_unit::SelectedUnitResource;
use crate::game::states::round_state::ActiveTeam;
use crate::game::team_setup::Team;
use crate::game::terrain::{MovementCost, Terrain};
use crate::game::unit_status::UnitStatus;

pub fn handle_selected_unit_input(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    buttons: Res<Input<MouseButton>>,
    #[allow(clippy::type_complexity)] mut units: Query<
        (
            Entity,
            &mut HexComponent,
            &Team,
            &mut ActionPoints,
            &UnitStatus,
            &CombatConfig,
        ),
        UnitFilter,
    >,
    hexes: Query<(&HexComponent, &Terrain), With<HexMarker>>,
    active_team: Res<ActiveTeam>,
    hovered_hex: Res<HoveredHex>,
    mut combat_event: EventWriter<CombatEvent>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(hex_cursor_position) = hovered_hex.0 else {
        return;
    };

    if let Some((hovered_entity, hovered_entity_hex, hovered_entity_team, _, _, _)) = units
        .iter()
        .find(|(_, hex, _, _, _, _)| hex.0 == hex_cursor_position)
    {
        if &active_team.0 == hovered_entity_team {
            selected_unit_resource.set_selected_unit(Some(hovered_entity));
            return;
        }

        let Some(selected_unit) = selected_unit_resource.selected_unit() else {
            return;
        };

        let (_, selected_unit_hex, _, action_points, _, combat_config) =
            units.get(selected_unit).unwrap();
        let distance = hovered_entity_hex
            .0
            .unsigned_distance_to(selected_unit_hex.0);
        if distance <= combat_config.range && action_points.can_still_attack_this_turn() {
            combat_event.send(CombatEvent {
                attacker: selected_unit,
                defender: hovered_entity,
            });

            let (_, _, _, mut action_points, _, _) = units.get_mut(selected_unit).unwrap();
            action_points.left -= action_points.attack_action_point_cost();
            action_points.attacks_this_round += 1;
        }
        return;
    }

    if let Some(selected_unit) = selected_unit_resource.selected_unit() {
        let Some(reachable_hexes) = selected_unit_resource.reachable_hexes() else {
            warn!(
                "Ignoring click for selected unit {selected_unit:?}, because had no reachable hexes"
            );
            return;
        };

        move_selected_unit_to_clicked_hex(
            &selected_unit_resource,
            &mut units,
            hexes,
            &hex_cursor_position,
            selected_unit,
            reachable_hexes,
            combat_event,
        )
    };
}

fn move_selected_unit_to_clicked_hex(
    selected_unit_resource: &SelectedUnitResource,
    #[allow(clippy::type_complexity)] units: &mut Query<
        (
            Entity,
            &mut HexComponent,
            &Team,
            &mut ActionPoints,
            &UnitStatus,
            &CombatConfig,
        ),
        UnitFilter,
    >,
    hexes: Query<(&HexComponent, &Terrain), With<HexMarker>>,
    hex_cursor_position: &Hex,
    selected_unit: Entity,
    reachable_hexes: &HashSet<Hex>,
    mut combat_event: EventWriter<CombatEvent>,
) {
    if !reachable_hexes.contains(&hex_cursor_position) {
        debug!("Reachable hexes don't contain cursor position {hex_cursor_position:?}");
        return;
    }

    let (_, mut hex_component, _, mut action_points, unit_status, _) =
        units.get_mut(selected_unit).unwrap();

    let Some(mut hexes_way) = a_star(hex_component.0, *hex_cursor_position, |hex| {
        selected_unit_resource
            .cost_map()
            .get(&hex)
            .and_then(|movement_cost| movement_cost.get_modified_algorithm_cost())
    }) else {
        warn!("A Star algorithm returned None");
        return;
    };

    hexes_way.remove(0);

    let cost: usize = hexes_way
        .into_iter()
        .map(|hex| {
            hexes
                .iter()
                .find(|(hex_component, _)| hex_component.0 == hex)
                .unwrap()
        })
        .map(|(_, terrain)| &terrain.movement_cost)
        .map(|movement_cost| match movement_cost {
            MovementCost::Impassable => {
                unreachable!("An impassable tile must not be on the way")
            }
            MovementCost::Passable(tile_cost) => tile_cost,
        })
        .sum();

    hex_component.0 = *hex_cursor_position;
    action_points.left -= cost;

    for unit_engaged_with in unit_status.get_engaged_with_units() {
        info!("{selected_unit:?} disengages from {unit_engaged_with:?} triggering attack");
        combat_event.send(CombatEvent {
            attacker: *unit_engaged_with,
            defender: selected_unit,
        });
    }
}

pub fn update_hovered_unit(
    mut hovered_unit_resource: ResMut<HoveredUnitResource>,
    units: Query<(Entity, &HexComponent), UnitFilter>,
    hovered_hex: Res<HoveredHex>,
) {
    let Some(hex_cursor_position) = hovered_hex.0 else {
        hovered_unit_resource.0 = None;
        return;
    };

    if let Some((hovered_entity, _)) = units.iter().find(|(_, hex)| hex.0 == hex_cursor_position) {
        hovered_unit_resource.0 = Some(hovered_entity);
        return;
    }
    hovered_unit_resource.0 = None;
}
