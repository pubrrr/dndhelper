use bevy::prelude::{info, warn, Event, EventReader, EventWriter, Query, Res, With};

use crate::game::ingame::action_points::ActionPoints;
use crate::game::ingame::combat::CombatEvent;
use crate::game::ingame::common_components::UnitFilter;
use crate::game::ingame::hex::{HexComponent, HexMarker};
use crate::game::ingame::path::CurrentPath;
use crate::game::ingame::selected_unit::SelectedUnitResource;
use crate::game::ingame::terrain::{MovementCost, Terrain};
use crate::game::ingame::unit_status::UnitStatus;

#[derive(Event, Debug)]
pub struct MoveUnitEvent;

pub(super) fn handle_move_event(
    mut move_events: EventReader<MoveUnitEvent>,
    selected_unit_resource: Res<SelectedUnitResource>,
    current_path: Res<CurrentPath>,
    mut combat_event: EventWriter<CombatEvent>,
    hexes: Query<(&HexComponent, &Terrain), With<HexMarker>>,
    mut units: Query<(&mut HexComponent, &mut ActionPoints, &UnitStatus), UnitFilter>,
) {
    let Some(move_event) = move_events.iter().next() else {
        return;
    };

    let Some(selected_unit) = selected_unit_resource.selected_unit() else {
        warn!("No selected unit for {move_event:?} - skipping it");
        return;
    };
    let Some(path_hexes) = &current_path.0 else {
        warn!("No current path for {move_event:?} - skipping it");
        return;
    };

    let cost: usize = path_hexes
        .iter()
        .skip(1)
        .map(|hex| {
            hexes
                .iter()
                .find(|(hex_component, _)| &hex_component.0 == hex)
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

    let (mut hex_component, mut action_points, unit_status) = units.get_mut(selected_unit).unwrap();

    hex_component.0 = *path_hexes.last().unwrap();
    action_points.left = if cost > action_points.left {
        0
    } else {
        action_points.left - cost
    };

    for unit_engaged_with in unit_status.get_engaged_with_units() {
        info!("{selected_unit:?} disengages from {unit_engaged_with:?} triggering attack");
        combat_event.send(CombatEvent {
            attacker: *unit_engaged_with,
            defender: selected_unit,
        });
    }
}
