use bevy::prelude::{NextState, ResMut, Resource, State, States};

use crate::game::selected_unit::SelectedUnitResource;
use crate::game::team_setup::Team;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum RoundState {
    #[default]
    Paused,
    Moving,
    Combat,
    RoundEnd,
}

#[derive(Resource)]
pub struct ActiveTeam(pub Team);

impl Default for ActiveTeam {
    fn default() -> Self {
        ActiveTeam(Team::Red)
    }
}

pub fn start_round_system(
    round_state: ResMut<State<RoundState>>,
    mut next_round_state: ResMut<NextState<RoundState>>,
) {
    if round_state.get() == &RoundState::Paused {
        next_round_state.set(RoundState::Moving);
    }
}

pub fn round_end_system(
    mut round_state: ResMut<NextState<RoundState>>,
    mut active_team: ResMut<ActiveTeam>,
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
) {
    selected_unit_resource.set_selected_unit(None);

    let next_team = match active_team.0 {
        Team::Red => Team::Blue,
        Team::Blue => Team::Red,
    };

    active_team.0 = next_team;

    round_state.set(RoundState::Moving);
}
