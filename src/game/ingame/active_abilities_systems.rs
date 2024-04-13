use bevy::prelude::{
    info_once, Commands, Entity, EventWriter, NextState, Parent, Query, ResMut, With,
};

use crate::game::abilities::active_abilities::{ActivatedAbilityMarker, ActiveAbility};
use crate::game::ingame::combat::CombatEvent;
use crate::game::ingame::selected_unit::SelectedUnitResource;
use crate::game::states::round_state::RoundState;

pub fn handle_activated_active_ability(
    mut round_state: ResMut<NextState<RoundState>>,
    active_abilities: Query<(&ActiveAbility, &Parent), With<ActivatedAbilityMarker>>,
    mut _combat_event: EventWriter<CombatEvent>,
) {
    let Ok((ability, parent)) = active_abilities.get_single() else {
        round_state.set(RoundState::Input);
        return;
    };

    info_once!("Handling activated ability {ability:?} for unit {parent:?}");

    // match ability {
    //     ActiveAbility::ThrowJavelin => combat_event.send(CombatEvent {
    //         attack: Attack {
    //             damage: 1,
    //             passive_combat_abilities: vec![],
    //             range: 2,
    //         },
    //         attacker: **parent,
    //         defender: todo!(),
    //     }),
    // };
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
