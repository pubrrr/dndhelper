use bevy::prelude::{Entity, Query, Res, ResMut};

use crate::clicked_hex::ClickedHex;
use crate::common_components::UnitFilter;
use crate::game_state::ActiveTeam;
use crate::hex::HexComponent;
use crate::team_setup::Team;
use crate::SelectedUnitResource;

pub fn handle_input(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    mut units: Query<(Entity, &mut HexComponent, &Team), UnitFilter>,
    active_team: Res<ActiveTeam>,
    clicked_hex: Res<ClickedHex>,
) {
    if let Some(hex_cursor_position) = clicked_hex.0 {
        if let Some((entity, _, team)) = units
            .iter()
            .find(|(_, hex, _)| hex.0 == hex_cursor_position)
        {
            if &active_team.0 == team {
                selected_unit_resource.selected_unit = Some(entity);
            }
            return;
        }

        if let Some(selected_unit) = selected_unit_resource.selected_unit {
            let (_, mut hex, _) = units.get_mut(selected_unit).unwrap();
            hex.0 = hex_cursor_position;
        }
    }
}
