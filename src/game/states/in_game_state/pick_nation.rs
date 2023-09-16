use bevy::app::App;
use bevy::prelude::EventReader;
#[cfg(not(test))]
use bevy::prelude::Update;
use bevy::prelude::{
    in_state, info, Commands, Event, IntoSystemConfigs, NextState, Plugin, PostUpdate, ResMut,
    Resource,
};

use crate::game::asset_loading::nation_asset_resource::NationKey;
use crate::game::ingame::team_setup::Team;
#[cfg(not(test))]
use crate::game::states::in_game_state::pick_nation::ui::pick_nation_menu;
use crate::game::states::in_game_state::InGameState;

pub(super) struct PickNationPlugin;

impl Plugin for PickNationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickNationEvent>();

        #[cfg(not(test))]
        app.add_systems(
            Update,
            pick_nation_menu.run_if(in_state(InGameState::PickNation)),
        );
        app.add_systems(
            PostUpdate,
            handle_pick_nation_event.run_if(in_state(InGameState::PickNation)),
        );
    }
}

#[derive(Event, Debug)]
pub(super) struct PickNationEvent {
    pub player: Team,
    pub nation: NationKey,
}

#[derive(Resource, Debug)]
pub struct PlayerPickedNationResource {
    pub player: Team,
    pub nation: NationKey,
}

#[cfg(not(test))]
mod ui {
    use bevy::prelude::{EventWriter, Res};
    use bevy_egui::egui::Window;
    use bevy_egui::EguiContexts;

    use crate::game::asset_loading::nation_asset_resource::NationAssetsResource;
    use crate::game::states::in_game_state::pick_nation::PickNationEvent;
    use crate::game::states::round_state::ActiveTeam;

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
