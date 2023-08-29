use bevy::prelude::{NextState, ResMut, Resource, States};

use crate::team_setup::Team;
use crate::SelectedUnitResource;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum GameState {
    #[default]
    Round,
    RoundEnd,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum RoundState {
    #[default]
    Moving,
    Combat,
}

#[derive(Resource)]
pub struct ActiveTeam(pub Team);

impl Default for ActiveTeam {
    fn default() -> Self {
        ActiveTeam(Team::Red)
    }
}

pub fn round_end_system(
    mut game_state: ResMut<NextState<GameState>>,
    mut round_state: ResMut<NextState<RoundState>>,
    mut active_team: ResMut<ActiveTeam>,
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
) {
    selected_unit_resource.selected_unit = None;

    let next_team = match active_team.0 {
        Team::Red => Team::Blue,
        Team::Blue => Team::Red,
    };

    active_team.0 = next_team;

    game_state.set(GameState::Round);
    round_state.set(RoundState::Moving);
}
