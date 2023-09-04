use std::fmt::{Display, Formatter};

use bevy::prelude::{default, Commands, Component, Res, SpriteBundle, Transform, Vec3};
use hexx::Hex;

use crate::game::action_points::ActionPoints;
use crate::game::combat::CombatConfig;
use crate::game::combat::HealthPoints;
use crate::game::common_components::UnitMarker;
use crate::game::hex::HexComponent;
use crate::game::nation_asset_resource::NationAssetsResource;
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

pub fn setup_team_units(mut commands: Commands, nation_assets_resource: Res<NationAssetsResource>) {
    let nations = nation_assets_resource.get_nations();

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
