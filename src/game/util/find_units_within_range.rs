use hexx::Hex;
use std::collections::HashSet;

use crate::game::ingame::selected_unit::UpdateReachableHexesUnitsQuery;
use crate::game::ingame::team_setup::Team;

pub trait FindUnitsWithinRange {
    fn find_units_within_range(
        &self,
        origin: Hex,
        range: u32,
        team_filter: impl Fn(&Team) -> bool,
    ) -> HashSet<Hex>;
}

impl FindUnitsWithinRange for UpdateReachableHexesUnitsQuery<'_, '_, '_> {
    fn find_units_within_range(
        &self,
        origin: Hex,
        range: u32,
        team_filter: impl Fn(&Team) -> bool,
    ) -> HashSet<Hex> {
        self.iter()
            .filter(|(_, _, team, _, _)| team_filter(team))
            .filter(|(_, hex_component, _, _, _)| {
                origin.unsigned_distance_to(hex_component.0) <= range
            })
            .map(|(_, hex_component, _, _, _)| hex_component.0)
            .collect()
    }
}
