use crate::game::states::in_game_state::InGameState;
use bevy::prelude::{info, NextState, Res, ResMut, State};

pub fn skip_events(
    in_game_state: Res<State<InGameState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if in_game_state.get() == &InGameState::Events {
        info!("Skipping events to {:?}", InGameState::DeployUnits);
        next_in_game_state.set(InGameState::DeployUnits);
    }
}
