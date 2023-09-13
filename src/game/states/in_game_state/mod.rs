use bevy::app::App;
use bevy::prelude::{
    in_state, IntoSystemConfigs, NextState, OnEnter, Plugin, PostUpdate, PreUpdate, ResMut,
    Resource, State, States, Update,
};
use bevy::utils::HashMap;

use crate::game::asset_loading::nation_asset_resource::NationKey;
use crate::game::ingame::hex::setup_hex_grid;
use crate::game::ingame::hovered_hex::update_hovered_hex;
use crate::game::ingame::post_update_systems::update_transform_from_hex;
use crate::game::ingame::team_setup::Team;
use crate::game::states::game_state::GameState;
use crate::game::states::in_game_state::deploy_units::DeployUnitsPlugin;
use crate::game::states::in_game_state::events::skip_events;
use crate::game::states::in_game_state::pick_commander::skip_pick_commander;
use crate::game::states::in_game_state::pick_nation::{
    handle_pick_nation_event, pick_nation_menu, PickNationEvent,
};
use crate::game::states::round_state::start_round_system;

mod deploy_units;
mod events;
mod pick_commander;
mod pick_nation;

pub struct StartupFlowPlugin;

impl Plugin for StartupFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeployUnitsPlugin)
            .add_state::<InGameState>()
            .add_event::<PickNationEvent>()
            .init_resource::<PickedNationsResource>()
            .add_systems(OnEnter(GameState::InGame), start_game)
            .add_systems(
                Update,
                pick_nation_menu.run_if(in_state(InGameState::PickNation)),
            )
            .add_systems(
                PostUpdate,
                handle_pick_nation_event.run_if(in_state(InGameState::PickNation)),
            )
            .add_systems(OnEnter(InGameState::PickCommander), skip_pick_commander)
            .add_systems(OnEnter(InGameState::Events), skip_events)
            .add_systems(OnEnter(InGameState::DeployUnits), setup_hex_grid)
            .add_systems(OnEnter(InGameState::Playing), start_round_system)
            .add_systems(
                PreUpdate,
                (update_transform_from_hex, update_hovered_hex)
                    .run_if(in_state(InGameState::DeployUnits)),
            );
    }
}

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
    pub nations_by_player: HashMap<Team, PickedNation>,
}

#[derive(Debug)]
pub struct PickedNation {
    pub nation: NationKey,
    // commander:
}

pub fn start_game(
    in_game_state: ResMut<State<InGameState>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if in_game_state.get() == &InGameState::Starting {
        next_in_game_state.set(InGameState::PickNation);
    }
}
