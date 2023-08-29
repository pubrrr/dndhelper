use bevy::prelude::{NextState, Res, ResMut};
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

use crate::game_state::{ActiveTeam, GameState};

pub fn ui_system(
    mut contexts: EguiContexts,
    active_team: Res<ActiveTeam>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    Window::new("Round").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Round of {}", active_team.0));
        if ui.button("End Round").clicked() {
            game_state.set(GameState::RoundEnd);
        };
    });
}
