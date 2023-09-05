pub mod deploy_units;
pub mod events;
pub mod pick_commander;
pub mod pick_nation;

use crate::game::nation_asset_resource::NationKey;
use crate::game::team_setup::Team;
use bevy::prelude::{NextState, ResMut, Resource, States};
use bevy::utils::HashMap;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum InGameState {
    #[default]
    Starting,
    PickNation,
    PickCommander,
    Events,
    DeployUnits,
    Playing,
}

#[derive(Resource, Debug, Default)]
pub struct PickedNationsResource {
    nations_by_player: HashMap<Team, PickedNation>,
}

#[derive(Debug)]
pub struct PickedNation {
    nation: NationKey,
    // commander:
}

pub fn start_game(mut in_game_state: ResMut<NextState<InGameState>>) {
    in_game_state.set(InGameState::PickNation)
}
