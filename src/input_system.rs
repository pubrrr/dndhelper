use bevy::prelude::{debug, warn, Entity, Input, MouseButton, NextState, Query, Res, ResMut, With};
use hexx::algorithms::a_star;

use crate::action_points::ActionPoints;
use crate::combat::CombatantsResource;
use crate::common_components::UnitFilter;
use crate::game_state::{ActiveTeam, RoundState};
use crate::hex::{HexComponent, HexMarker};
use crate::hovered_hex::HoveredHex;
use crate::hovered_hex::HoveredUnitResource;
use crate::selected_unit::SelectedUnitResource;
use crate::team_setup::Team;
use crate::terrain::{MovementCost, Terrain};

pub fn handle_selected_unit_input(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    buttons: Res<Input<MouseButton>>,
    mut units: Query<(Entity, &mut HexComponent, &Team, &mut ActionPoints), UnitFilter>,
    hexes: Query<(&HexComponent, &Terrain), With<HexMarker>>,
    active_team: Res<ActiveTeam>,
    hovered_hex: Res<HoveredHex>,
    mut next_round_state: ResMut<NextState<RoundState>>,
    mut combatants_resource: ResMut<CombatantsResource>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(hex_cursor_position) = hovered_hex.0 else {
        return;
    };

    if let Some((hovered_entity, hovered_entity_hex, hovered_entity_team, _)) = units
        .iter()
        .find(|(_, hex, _, _)| hex.0 == hex_cursor_position)
    {
        if &active_team.0 == hovered_entity_team {
            selected_unit_resource.set_selected_unit(Some(hovered_entity));
            return;
        }

        let Some(selected_unit) = selected_unit_resource.selected_unit() else {
            return;
        };

        let (_, selected_unit_hex, _, _) = units.get(selected_unit).unwrap();
        let distance = hovered_entity_hex
            .0
            .unsigned_distance_to(selected_unit_hex.0);
        if distance == 1 {
            next_round_state.set(RoundState::Combat);
            *combatants_resource = CombatantsResource::Combatants {
                attacker: selected_unit,
                defender: hovered_entity,
            }
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

        if !reachable_hexes.contains(&hex_cursor_position) {
            debug!("Reachable hexes don't contain cursor position {hex_cursor_position:?}");
            return;
        }

        let (_, mut hex_component, _, mut action_points) = units.get_mut(selected_unit).unwrap();

        let Some(mut hexes_way) = a_star(hex_component.0, hex_cursor_position, |hex| {
            selected_unit_resource
                .cost_map()
                .get(&hex)
                .and_then(|movement_cost| movement_cost.get_modified_algorithm_cost())
        }) else {
            debug!("A Star algorithm returned None");
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

        hex_component.0 = hex_cursor_position;
        action_points.left -= cost;
    };
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
