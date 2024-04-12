use crate::game::abilities::active_abilities::{ActivatedAbilityMarker, ActiveAbility};
use crate::game::states::round_state::RoundState;
use bevy::prelude::{info, NextState, Query, ResMut, With};

pub fn handle_activated_active_ability(
    mut round_state: ResMut<NextState<RoundState>>,
    active_abilities: Query<&ActiveAbility, With<ActivatedAbilityMarker>>,
) {
    info!("{:?}", active_abilities.single());
    round_state.set(RoundState::Input);
}
