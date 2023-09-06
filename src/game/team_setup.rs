use std::fmt::{Display, Formatter};

use bevy::prelude::{
    default, Commands, Component, NextState, Res, ResMut, SpriteBundle, States, Transform, Vec3,
};
use hexx::Hex;

use crate::game::action_points::ActionPoints;
use crate::game::combat::CombatConfig;
use crate::game::combat::HealthPoints;
use crate::game::common_components::UnitMarker;
use crate::game::hex::HexComponent;
use crate::game::nation_asset_resource::NationAssetsResource;
use crate::game::states::in_game_state::PickedNation;
use crate::game::states::in_game_state::{InGameState, PickedNationsResource};
use crate::game::states::round_state::RoundState;
use crate::game::unit_status::UnitStatus;
use crate::game::z_ordering::ZOrdering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum Team {
    Red,
    Blue,
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Team::Red => write!(f, "Red"),
            Team::Blue => write!(f, "Blue"),
        }
    }
}

#[derive(States, Hash, Eq, PartialEq, Default, Debug, Clone)]
pub enum QuickstartState {
    #[default]
    None,
    DoIt,
}

pub fn quickstart(
    mut commands: Commands,
    nation_assets_resource: Res<NationAssetsResource>,
    mut picked_nations: ResMut<PickedNationsResource>,
    mut in_game_state: ResMut<NextState<InGameState>>,
    mut round_state: ResMut<NextState<RoundState>>,
) {
    in_game_state.set(InGameState::Playing);
    round_state.set(RoundState::Moving);

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

    let manf_unit_key = &nation_assets_resource.get_units(&nations[1].key)[0];
    let manf_assets = nation_assets_resource.get_unit_assets(manf_unit_key);
    let tree_unit_key = &nation_assets_resource.get_units(&nations[0].key)[0];
    let tree_assets = nation_assets_resource.get_unit_assets(tree_unit_key);
    for i in 0..5 {
        commands
            .spawn(SpriteBundle {
                texture: manf_assets.image.clone(),
                transform: Transform::from_xyz(0., 0., ZOrdering::UNITS)
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(Team::Red)
            .insert(UnitStatus::new())
            .insert(UnitMarker(manf_assets.stats.name.clone()))
            .insert(ActionPoints::new(
                manf_assets.stats.max_action_points,
                manf_assets.stats.max_attacks_per_round,
                manf_assets.stats.attack_action_point_cost,
            ))
            .insert(HealthPoints::new(manf_assets.stats.max_health_points))
            .insert(CombatConfig {
                attack: manf_assets.stats.attack,
                defense: manf_assets.stats.defense,
            })
            .insert(HexComponent(Hex::new(4, i - 4)));

        commands
            .spawn(SpriteBundle {
                texture: tree_assets.image.clone(),
                transform: Transform::from_xyz(0., 0., ZOrdering::UNITS)
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(UnitMarker(tree_assets.stats.name.clone()))
            .insert(Team::Blue)
            .insert(UnitStatus::new())
            .insert(ActionPoints::new(
                tree_assets.stats.max_action_points,
                tree_assets.stats.max_attacks_per_round,
                tree_assets.stats.attack_action_point_cost,
            ))
            .insert(HealthPoints::new(tree_assets.stats.max_health_points))
            .insert(CombatConfig {
                attack: tree_assets.stats.attack,
                defense: tree_assets.stats.defense,
            })
            .insert(HexComponent(Hex::new(-4, i)));
    }
}
