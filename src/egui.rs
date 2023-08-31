use bevy::prelude::{NextState, Query, Res, ResMut};
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

use crate::action_points::ActionPoints;
use crate::combat::HealthPoints;
use crate::game_state::{ActiveTeam, GameState};
use crate::team_setup::Team;
use crate::{HoveredUnitResource, SelectedUnitResource};

pub fn ui_system(
    mut contexts: EguiContexts,
    active_team: Res<ActiveTeam>,
    mut game_state: ResMut<NextState<GameState>>,
    selected_unit_resource: Res<SelectedUnitResource>,
    hovered_unit_resource: Res<HoveredUnitResource>,
    units: Query<(&ActionPoints, &HealthPoints, &Team)>,
) {
    Window::new("Round").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Round of {}", active_team.0));

        let unit_to_show = hovered_unit_resource
            .0
            .or(selected_unit_resource.selected_unit);
        if let Some(selected_unit) = unit_to_show {
            let (action_points, health_points, team) = units.get(selected_unit).unwrap();
            ui.label(format!("Owner: {team}"));
            ui.label(format!(
                "Action points: {}/{}",
                action_points.left,
                action_points.get_max()
            ));
            ui.label(format!(
                "Health points: {}/{}",
                health_points.left,
                health_points.get_max()
            ));
        } else {
            ui.label("-");
        }

        if ui.button("End Round").clicked() {
            game_state.set(GameState::RoundEnd);
        };
    });
}
