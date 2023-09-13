use bevy::prelude::{info, Commands, NextState, Res, ResMut, State};

use crate::game::ingame::team_setup::Team;
use crate::game::states::in_game_state::pick_nation::PlayerPickedNationResource;
use crate::game::states::in_game_state::{InGameState, PickedNation, PickedNationsResource};
use crate::game::states::round_state::ActiveTeam;

pub(super) fn skip_pick_commander(
    mut commands: Commands,
    in_game_state: Res<State<InGameState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    player_picked_nations_resource: Res<PlayerPickedNationResource>,
    mut picked_nations_resource: ResMut<PickedNationsResource>,
    mut active_player: ResMut<ActiveTeam>,
) {
    if in_game_state.get() != &InGameState::PickCommander {
        return;
    }

    picked_nations_resource.nations_by_player.insert(
        player_picked_nations_resource.player,
        PickedNation {
            nation: player_picked_nations_resource.nation.clone(),
        },
    );

    let next_team = match active_player.0 {
        Team::Red => Team::Blue,
        Team::Blue => Team::Red,
    };

    active_player.0 = next_team;
    commands.remove_resource::<PlayerPickedNationResource>();

    if picked_nations_resource.nations_by_player.len() >= 2 {
        next_in_game_state.set(InGameState::Events);
        info!("Continuing to {:?}", InGameState::Events);
        return;
    }
    info!("Continuing to {:?}", InGameState::PickNation);
    next_in_game_state.set(InGameState::PickNation);
}
