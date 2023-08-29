use bevy::prelude::{Entity, Query, Res, ResMut};

use crate::action_points::ActionPoints;
use crate::clicked_hex::ClickedHex;
use crate::common_components::UnitFilter;
use crate::game_state::ActiveTeam;
use crate::hex::HexComponent;
use crate::team_setup::Team;
use crate::SelectedUnitResource;

pub fn handle_input(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    mut units: Query<(Entity, &mut HexComponent, &Team, &mut ActionPoints), UnitFilter>,
    active_team: Res<ActiveTeam>,
    clicked_hex: Res<ClickedHex>,
) {
    if let Some(hex_cursor_position) = clicked_hex.0 {
        if let Some((entity, _, team, _)) = units
            .iter()
            .find(|(_, hex, _, _)| hex.0 == hex_cursor_position)
        {
            if &active_team.0 == team {
                selected_unit_resource.selected_unit = Some(entity);
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
}
