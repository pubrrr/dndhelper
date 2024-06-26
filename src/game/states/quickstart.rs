use bevy::app::App;
use bevy::prelude::{
    default, Commands, NextState, OnEnter, Plugin, Res, ResMut, SpriteBundle, States, Transform,
    Vec3,
};
use hexx::Hex;

use crate::game::asset_loading::nation_asset_resource::NationAssetsResource;
use crate::game::ingame::action_points::ActionPoints;
use crate::game::ingame::combat::{CombatConfig, HealthPoints};
use crate::game::ingame::hex::{setup_hex_grid, HexComponent};
use crate::game::ingame::team_setup::Team;
use crate::game::ingame::unit::UnitMarker;
use crate::game::ingame::unit_status::UnitStatus;
use crate::game::ingame::z_ordering::ZOrdering;
use crate::game::states::in_game_state::{InGameState, PickedNation, PickedNationsResource};
use crate::game::states::round_state::RoundState;

pub struct QuickstartPlugin;

impl Plugin for QuickstartPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<QuickstartState>()
            .add_systems(OnEnter(QuickstartState::DoIt), (setup_hex_grid, quickstart));
    }
}

#[derive(States, Hash, Eq, PartialEq, Default, Debug, Clone)]
pub enum QuickstartState {
    #[default]
    None,
    DoIt,
}

fn quickstart(
    mut commands: Commands,
    nation_assets_resource: Res<NationAssetsResource>,
    mut picked_nations: ResMut<PickedNationsResource>,
    mut in_game_state: ResMut<NextState<InGameState>>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    in_game_state.set(InGameState::Playing);
    round_state.set(RoundState::Input);

    let nations = nation_assets_resource.get_nations();

    picked_nations.nations_by_player.insert(
        Team::Red,
        PickedNation {
            nation: nations[1].key.clone(),
        },
    );
    picked_nations.nations_by_player.insert(
        Team::Blue,
        PickedNation {
            nation: nations[0].key.clone(),
        },
    );

    let manf_unit_keys = &nation_assets_resource.get_units(&nations[1].key);
    let tree_unit_keys = &nation_assets_resource.get_units(&nations[0].key);
    for i in 0..5 {
        let manf_assets =
            nation_assets_resource.get_unit_assets(&manf_unit_keys[i % manf_unit_keys.len()]);
        let tree_assets =
            nation_assets_resource.get_unit_assets(&tree_unit_keys[i % tree_unit_keys.len()]);

        commands
            .spawn(SpriteBundle {
                texture: manf_assets.image.clone(),
                transform: Transform::from_xyz(0., 0., ZOrdering::UNITS)
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(Team::Red)
            .insert(UnitStatus::default())
            .insert(UnitMarker(manf_assets.stats.name.clone()))
            .insert(ActionPoints::new(
                manf_assets.stats.max_action_points,
                manf_assets.stats.max_attacks_per_round,
                manf_assets.stats.attack_action_point_cost,
            ))
            .insert(HealthPoints::new(manf_assets.stats.max_health_points))
            .insert(CombatConfig {
                damage: manf_assets.stats.damage,
                defense: manf_assets.stats.defense,
                range: manf_assets.stats.range,
                passive_combat_abilities: vec![],
            })
            .insert(HexComponent(Hex::new(4, i as i32 - 4)));

        commands
            .spawn(SpriteBundle {
                texture: tree_assets.image.clone(),
                transform: Transform::from_xyz(0., 0., ZOrdering::UNITS)
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(UnitMarker(tree_assets.stats.name.clone()))
            .insert(Team::Blue)
            .insert(UnitStatus::default())
            .insert(ActionPoints::new(
                tree_assets.stats.max_action_points,
                tree_assets.stats.max_attacks_per_round,
                tree_assets.stats.attack_action_point_cost,
            ))
            .insert(HealthPoints::new(tree_assets.stats.max_health_points))
            .insert(CombatConfig {
                damage: tree_assets.stats.damage,
                defense: tree_assets.stats.defense,
                range: tree_assets.stats.range,
                passive_combat_abilities: vec![],
            })
            .insert(HexComponent(Hex::new(-4, i as i32)));
    }
}
