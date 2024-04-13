use bevy::app::App;
use bevy::prelude::{NextState, OnEnter, Plugin, ResMut, Resource, State, States};
use bevy::utils::HashMap;

use crate::game::asset_loading::nation_asset_resource::NationKey;
use crate::game::ingame::team_setup::Team;
use crate::game::states::game_state::GameState;
use crate::game::states::in_game_state::deploy_units::DeployUnitsPlugin;
use crate::game::states::in_game_state::events::skip_events;
use crate::game::states::in_game_state::pick_commander::skip_pick_commander;
use crate::game::states::in_game_state::pick_nation::{PickNationEvent, PickNationPlugin};
use crate::game::states::round_state::start_round_system;

mod deploy_units;
mod events;
mod pick_commander;
mod pick_nation;

pub struct StartupFlowPlugin;

impl Plugin for StartupFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PickNationPlugin, DeployUnitsPlugin))
            .init_state::<InGameState>()
            .add_event::<PickNationEvent>()
            .init_resource::<PickedNationsResource>()
            .add_systems(OnEnter(GameState::InGame), start_game)
            .add_systems(OnEnter(InGameState::PickCommander), skip_pick_commander)
            .add_systems(OnEnter(InGameState::Events), skip_events)
            .add_systems(OnEnter(InGameState::Playing), start_round_system);
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

#[cfg(test)]
mod tests {
    use bevy::input::mouse::MouseButtonInput;
    use bevy::input::{ButtonState, InputPlugin};
    use bevy::prelude::{AssetApp, AssetPlugin, ColorMaterial, Entity, Handle, Mesh, MouseButton};
    use hexx::Hex;

    use crate::game::asset_loading::nation_asset_resource::NationAssetsResource;
    use crate::game::asset_loading::nation_assets::{UnitKey, UnitStats};
    use crate::game::ingame::hex::HexComponent;
    use crate::game::ingame::hovered_hex::HoveredHex;
    use crate::game::ingame::unit::UnitMarker;
    use crate::game::states::in_game_state::deploy_units::{
        DeploymentDoneEvent, SelectedUnitToDeploy,
    };
    use crate::game::states::round_state::{ActiveTeam, RoundState};
    use crate::generate_test_app;
    use crate::tests::AppWrapper;

    use super::*;

    const NATION_1: &str = "nation 1";
    const UNIT_1: &str = "unit 1";

    const NATION_2: &str = "nation 2";
    const UNIT_2: &str = "unit 2";

    #[test]
    fn startup_flow() {
        let mut app = TestApp::init();

        app.pick_nation(Team::Red, NationKey(NATION_1.to_string()));
        app.pick_nation(Team::Blue, NationKey(NATION_2.to_string()));

        assert_eq!(
            app.get_picked_nation(Team::Red),
            NationKey(NATION_1.to_string())
        );
        assert_eq!(
            app.get_picked_nation(Team::Blue),
            NationKey(NATION_2.to_string())
        );

        app.update();
        app.update();

        let unit_1_key = UnitKey {
            nation: NATION_1.to_string(),
            name: UNIT_1.to_string(),
        };
        app.deploy_unit_at(unit_1_key, Hex::new(2, 0));

        let units = app.get_units();
        assert_eq!(units.len(), 1);
        let (unit_marker, hex_component, team) = units[0];
        assert_eq!(unit_marker, &UnitMarker(UNIT_1.to_string()));
        assert_eq!(hex_component.0, Hex::new(2, 0));
        assert_eq!(team, &Team::Red);

        app.send_event(DeploymentDoneEvent);

        app.update();

        let unit_2_key = UnitKey {
            nation: NATION_2.to_string(),
            name: "unit 2".to_string(),
        };
        app.deploy_unit_at(unit_2_key, Hex::new(-2, 0));

        let units = app.get_units();
        assert_eq!(units.len(), 2);
        assert!(units.iter().any(|(_, _, team)| team == &&Team::Red));
        let (unit_marker, hex_component, _) = units
            .iter()
            .find(|(_, _, team)| team == &&Team::Blue)
            .unwrap();
        assert_eq!(unit_marker, &&UnitMarker(UNIT_2.to_string()));
        assert_eq!(hex_component.0, Hex::new(-2, 0));

        app.send_event(DeploymentDoneEvent);

        app.update();
        app.update();

        assert_eq!(app.get_ingame_state(), &InGameState::Playing);
    }

    generate_test_app!();

    impl TestApp {
        fn init() -> Self {
            let mut app = App::new();

            let unit_key_1 = UnitKey {
                nation: NATION_1.to_string(),
                name: UNIT_1.to_string(),
            };
            let unit_key_2 = UnitKey {
                nation: NATION_2.to_string(),
                name: UNIT_2.to_string(),
            };

            app.init_state::<GameState>();
            app.init_state::<RoundState>();
            app.insert_resource(NationAssetsResource {
                nation_assets_definition: vec![],
                unit_images: HashMap::from([
                    (unit_key_1.get_image_asset_path(), Handle::default()),
                    (unit_key_2.get_image_asset_path(), Handle::default()),
                ]),
                unit_stats: HashMap::from([
                    (
                        unit_key_1.get_stats_asset_path(),
                        UnitStats {
                            name: UNIT_1.to_string(),
                            max_action_points: 0,
                            max_health_points: 0,
                            damage: 0,
                            defense: 0,
                            attack_action_point_cost: 0,
                            max_attacks_per_round: 0,
                            range: 0,
                            passive_combat_abilities: vec![],
                            active_abilities: vec![],
                        },
                    ),
                    (
                        unit_key_2.get_stats_asset_path(),
                        UnitStats {
                            name: UNIT_2.to_string(),
                            max_action_points: 0,
                            max_health_points: 0,
                            damage: 0,
                            defense: 0,
                            attack_action_point_cost: 0,
                            max_attacks_per_round: 0,
                            range: 0,
                            passive_combat_abilities: vec![],
                            active_abilities: vec![],
                        },
                    ),
                ]),
            });
            app.init_resource::<ActiveTeam>();
            app.init_resource::<HoveredHex>();
            app.add_plugins((AssetPlugin::default(), InputPlugin, StartupFlowPlugin));
            app.init_asset::<Mesh>();
            app.init_asset::<ColorMaterial>();

            app.world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::InGame);

            app.update();

            Self { app }
        }

        fn set_unit_to_deploy(&mut self, unit: UnitKey) {
            self.app.world.resource_mut::<SelectedUnitToDeploy>().0 = Some(unit);
        }

        fn set_hovered_hex(&mut self, hex: Hex) {
            self.app.world.resource_mut::<HoveredHex>().0 = Some(hex);
        }

        fn get_picked_nation(&self, player: Team) -> NationKey {
            self.app
                .world
                .resource::<PickedNationsResource>()
                .nations_by_player[&player]
                .nation
                .clone()
        }

        fn pick_nation(&mut self, player: Team, nation: NationKey) {
            self.send_event(PickNationEvent { player, nation });
            self.app.update();
            self.app.update();
        }

        fn deploy_unit_at(&mut self, unit: UnitKey, hex: Hex) {
            self.set_unit_to_deploy(unit);
            self.set_hovered_hex(hex);
            self.left_click();
            self.app.update();
        }

        fn left_click(&mut self) {
            self.send_event(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Released,
                window: Entity::from_raw(0),
            });
            self.send_event(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
                window: Entity::from_raw(0),
            });
        }

        fn get_units(&mut self) -> Vec<(&UnitMarker, &HexComponent, &Team)> {
            self.app
                .world
                .query::<(&UnitMarker, &HexComponent, &Team)>()
                .iter(&self.app.world)
                .collect()
        }

        fn get_ingame_state(&mut self) -> &InGameState {
            self.app.world.resource::<State<InGameState>>().get()
        }
    }
}
