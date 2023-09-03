use crate::game_state::GameState;
use crate::nation_assets::LoadingState;
use bevy::prelude::{NextState, Res, ResMut, State};
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

pub fn menu_ui(
    mut contexts: EguiContexts,

    loading_state: Res<State<LoadingState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    Window::new("Menu").show(contexts.ctx_mut(), |ui| match loading_state.get() {
        LoadingState::Loading => {
            ui.label("Loading...");
        }
        LoadingState::Done => {
            if ui.button("Start").clicked() {
                next_game_state.set(GameState::InGame);
            }
        }
    });
}
