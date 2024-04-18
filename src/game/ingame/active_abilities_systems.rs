use bevy::input::ButtonInput;
use bevy::prelude::{
    info_once, warn, Commands, Entity, MouseButton, NextState, Parent, Query, Res, ResMut, With,
};

use crate::game::abilities::active_abilities::{
    ActivatedAbilityMarker, ActiveAbility, ThrowJavelinInput,
};
use crate::game::ingame::hovered_hex::HoveredHex;
use crate::game::ingame::selected_unit::SelectedUnitResource;
use crate::game::states::round_state::RoundState;

pub fn handle_activated_active_ability(
    selected_unit_resource: Res<SelectedUnitResource>,
    buttons: Res<ButtonInput<MouseButton>>,
    hovered_hex: Res<HoveredHex>,
    mut round_state: ResMut<NextState<RoundState>>,
    active_abilities: Query<(&ActiveAbility, &Parent), With<ActivatedAbilityMarker>>,
    mut commands: Commands,
) {
    let Ok((ability, parent)) = active_abilities.get_single() else {
        round_state.set(RoundState::Input);
        return;
    };

    info_once!("Handling activated ability {ability:?} for unit {parent:?}");

    let Some(reachable_hexes) = selected_unit_resource.reachable_hexes() else {
        warn!("No reachable hexes found for ability {ability:?} for unit {parent:?}");
        round_state.set(RoundState::Input);
        return;
    };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(hex_cursor_position) = hovered_hex.0 else {
        return;
    };

    if !reachable_hexes.contains(&hex_cursor_position) {
        return;
    }

    match ability {
        ActiveAbility::ThrowJavelin {
            throw_javelin_system: system_id,
        } => commands.run_system_with_input(
            *system_id,
            ThrowJavelinInput {
                attacker: **parent,
                target_hex: hex_cursor_position,
            },
        ),
    };
}

pub fn unset_activated_ability(
    active_abilities: Query<Entity, With<ActivatedAbilityMarker>>,
    mut commands: Commands,
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
) {
    for entity in active_abilities.iter() {
        commands.entity(entity).remove::<ActivatedAbilityMarker>();
    }
    selected_unit_resource.needs_reachable_hexes_recomputation();
}
