use bevy::prelude::{Entity, Input, MouseButton, NextState, Query, Res, ResMut};

use crate::action_points::ActionPoints;
use crate::combat::CombatantsResource;
use crate::common_components::UnitFilter;
use crate::game_state::{ActiveTeam, RoundState};
use crate::hex::HexComponent;
use crate::hovered_hex::HoveredHex;
use crate::selected_unit::SelectedUnitResource;
use crate::team_setup::Team;
use crate::HoveredUnitResource;

pub fn update_selected_unit(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    buttons: Res<Input<MouseButton>>,
    mut units: Query<(Entity, &mut HexComponent, &Team, &mut ActionPoints), UnitFilter>,
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
            selected_unit_resource.selected_unit = Some(hovered_entity);
            return;
        }

        let Some(selected_unit) = selected_unit_resource.selected_unit else {
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

    if let Some(selected_unit) = selected_unit_resource.selected_unit {
        let (_, mut hex, _, mut action_points) = units.get_mut(selected_unit).unwrap();
        let distance = hex.0.unsigned_distance_to(hex_cursor_position) as usize;
        if distance > action_points.left {
            return;
        }
        hex.0 = hex_cursor_position;
        action_points.left -= distance;
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
