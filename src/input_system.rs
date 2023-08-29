use bevy::log::info;
use bevy::prelude::{Entity, NextState, Query, Res, ResMut};

use crate::action_points::ActionPoints;
use crate::clicked_hex::ClickedHex;
use crate::combat::CombatantsResource;
use crate::common_components::UnitFilter;
use crate::game_state::{ActiveTeam, RoundState};
use crate::hex::HexComponent;
use crate::team_setup::Team;
use crate::SelectedUnitResource;

pub fn handle_input(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    mut units: Query<(Entity, &mut HexComponent, &Team, &mut ActionPoints), UnitFilter>,
    active_team: Res<ActiveTeam>,
    clicked_hex: Res<ClickedHex>,
    mut next_round_state: ResMut<NextState<RoundState>>,
    mut combatants_resource: ResMut<CombatantsResource>,
) {
    let Some(hex_cursor_position) = clicked_hex.0 else {
        return;
    };

    if let Some((clicked_entity, clicked_entity_hex, clicked_entity_team, _)) = units
        .iter()
        .find(|(_, hex, _, _)| hex.0 == hex_cursor_position)
    {
        if &active_team.0 == clicked_entity_team {
            selected_unit_resource.selected_unit = Some(clicked_entity);
            return;
        }

        let Some(selected_unit) = selected_unit_resource.selected_unit else {
            return;
        };

        let (_, selected_unit_hex, _, _) = units.get(selected_unit).unwrap();
        let distance = clicked_entity_hex
            .0
            .unsigned_distance_to(selected_unit_hex.0);
        if distance == 1 {
            next_round_state.set(RoundState::Combat);
            info!("attacker: {selected_unit:?}, defender: {clicked_entity:?}");
            *combatants_resource = CombatantsResource::Combatants {
                attacker: selected_unit,
                defender: clicked_entity,
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
