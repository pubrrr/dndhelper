use crate::action_points::ActionPoints;
use bevy::prelude::{NextState, Query, Res, ResMut};
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

use crate::game_state::{ActiveTeam, GameState};
use crate::SelectedUnitResource;

pub fn ui_system(
    mut contexts: EguiContexts,
    active_team: Res<ActiveTeam>,
    mut game_state: ResMut<NextState<GameState>>,
    selected_unit_resource: Res<SelectedUnitResource>,
    units: Query<&ActionPoints>,
) {
    Window::new("Round").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Round of {}", active_team.0));

        let action_points_display = match &selected_unit_resource.selected_unit {
            None => "Action points: -".to_string(),
            Some(selected_unit) => {
                let action_points = units.get(*selected_unit).unwrap();
                format!(
                    "Action points: {}/{}",
                    action_points.left,
                    action_points.get_max()
                )
            }
        };
        ui.label(action_points_display);

        if ui.button("End Round").clicked() {
            game_state.set(GameState::RoundEnd);
        };
    });
}
