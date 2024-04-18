use bevy::input::ButtonInput;
use bevy::prelude::{debug, warn, Entity, EventWriter, MouseButton, Query, Res, ResMut};

use crate::game::ingame::action_points::ActionPoints;
use crate::game::ingame::combat::{AttackOrDefault, CombatConfig, CombatEvent};
use crate::game::ingame::hex::HexComponent;
use crate::game::ingame::hovered_hex::{HoveredHex, HoveredUnitResource};
use crate::game::ingame::move_unit::MoveUnitEvent;
use crate::game::ingame::path::CurrentPath;
use crate::game::ingame::selected_unit::SelectedUnitResource;
use crate::game::ingame::team_setup::Team;
use crate::game::ingame::unit::UnitFilter;
use crate::game::states::round_state::ActiveTeam;

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(super) fn handle_selected_unit_input(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    buttons: Res<ButtonInput<MouseButton>>,
    #[allow(clippy::type_complexity)] mut units: Query<
        (
            Entity,
            &HexComponent,
            &Team,
            &mut ActionPoints,
            &CombatConfig,
        ),
        UnitFilter,
    >,
    active_team: Res<ActiveTeam>,
    hovered_hex: Res<HoveredHex>,
    current_path: Res<CurrentPath>,
    mut combat_event: EventWriter<CombatEvent>,
    mut move_event: EventWriter<MoveUnitEvent>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(hex_cursor_position) = hovered_hex.0 else {
        return;
    };

    if let Some((hovered_entity, hovered_entity_hex, hovered_entity_team, _, _)) = units
        .iter()
        .find(|(_, hex, _, _, _)| hex.0 == hex_cursor_position)
    {
        if &active_team.0 == hovered_entity_team {
            selected_unit_resource.set_selected_unit(Some(hovered_entity));
            return;
        }

        let Some(selected_unit) = selected_unit_resource.selected_unit() else {
            return;
        };

        let Ok((_, selected_unit_hex, _, action_points, combat_config)) = units.get(selected_unit)
        else {
            return;
        };

        let distance = hovered_entity_hex
            .0
            .unsigned_distance_to(selected_unit_hex.0);
        if distance <= combat_config.range && action_points.can_still_attack_this_turn() {
            combat_event.send(CombatEvent {
                attacker: selected_unit,
                defender: hovered_entity,
                attack: AttackOrDefault::Attack(combat_config.get_default_attack()),
            });

            let (_, _, _, mut action_points, _) = units.get_mut(selected_unit).unwrap();
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

        let Some(current_path) = &current_path.0 else {
            warn!(
                "Ignoring click for selected unit {selected_unit:?}, because had no current path"
            );
            return;
        };

        if !reachable_hexes.contains(&hex_cursor_position) {
            debug!("Reachable hexes don't contain cursor position {hex_cursor_position:?}");
            return;
        }

        move_event.send(MoveUnitEvent {
            entity: selected_unit,
            path: current_path.iter().cloned().skip(1).collect(),
        });
    };
}

pub(super) fn update_hovered_unit(
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
