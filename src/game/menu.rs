use crate::game::nation_assets::LoadingState;
use crate::game::states::game_state::GameState;
use crate::game::team_setup::QuickstartState;
use bevy::prelude::{NextState, Res, ResMut, State};
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

pub fn menu_ui(
    mut contexts: EguiContexts,
    loading_state: Res<State<LoadingState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_quickstart_state: ResMut<NextState<QuickstartState>>,
) {
    Window::new("Menu").show(contexts.ctx_mut(), |ui| match loading_state.get() {
        LoadingState::LoadingDynamicAssets | LoadingState::LoadingNationAssetsDefinition => {
            ui.label("Loading...");
        }
        LoadingState::Done => {
            if ui.button("Start").clicked() {
                next_game_state.set(GameState::InGame);
            }
            if ui.button("Quickstart").clicked() {
                next_game_state.set(GameState::InGame);
                next_quickstart_state.set(QuickstartState::DoIt);
            }
        }
    });
}
