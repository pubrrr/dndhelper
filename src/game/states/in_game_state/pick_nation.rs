use bevy::prelude::{
    info, Commands, Event, EventReader, EventWriter, NextState, Res, ResMut, Resource,
};
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

use crate::game::asset_loading::nation_asset_resource::{NationAssetsResource, NationKey};
use crate::game::ingame::team_setup::Team;
use crate::game::states::in_game_state::InGameState;
use crate::game::states::round_state::ActiveTeam;

#[derive(Event, Debug)]
pub struct PickNationEvent {
    player: Team,
    nation: NationKey,
}

#[derive(Resource, Debug)]
pub struct PlayerPickedNationResource {
    pub player: Team,
    pub nation: NationKey,
}

pub(super) fn pick_nation_menu(
    mut contexts: EguiContexts,
    mut pick_nation_event: EventWriter<PickNationEvent>,
    nation_assets_resource: Res<NationAssetsResource>,
    active_player: Res<ActiveTeam>,
) {
    Window::new("Pick Nation").show(contexts.ctx_mut(), |ui| {
        ui.heading(format!("{}", active_player.0));

        for nation in nation_assets_resource.get_nations() {
            if ui.button(nation.name).clicked() {
                pick_nation_event.send(PickNationEvent {
                    player: active_player.0,
                    nation: nation.key,
                });
            }
        }
    });
}

pub(super) fn handle_pick_nation_event(
    mut pick_nation_events: EventReader<PickNationEvent>,
    mut commands: Commands,
    mut in_game_state: ResMut<NextState<InGameState>>,
) {
    let Some(event) = pick_nation_events.iter().next() else {
        return;
    };

    in_game_state.set(InGameState::PickCommander);
    info!("{} picked nation {:?}", event.player, event.nation);
    commands.insert_resource(PlayerPickedNationResource {
        player: event.player,
        nation: event.nation.clone(),
    })
}
