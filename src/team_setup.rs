use std::fmt::{Display, Formatter};

use bevy::prelude::{default, Commands, Component, Res, SpriteBundle, Transform, Vec3};
use hexx::Hex;

use crate::action_points::ActionPoints;
use crate::combat::CombatConfig;
use crate::combat::HealthPoints;
use crate::common_components::UnitMarker;
use crate::hex::HexComponent;
use crate::nation_assets::NationAssetCollection;
use crate::z_ordering::ZOrdering;

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

pub fn setup_team_units(mut commands: Commands, unit_assets: Res<NationAssetCollection>) {
    let manf_image = &unit_assets.unit_images["nations/Mansreich/units/Manf/sprite.png"];
    let tree_image = &unit_assets.unit_images["nations/Kurzland/units/Tree/sprite.png"];
    for i in 0..5 {
        commands
            .spawn(SpriteBundle {
                texture: manf_image.clone(),
                transform: Transform::from_xyz(0., 0., ZOrdering::UNITS)
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(Team::Red)
            .insert(UnitMarker)
            .insert(ActionPoints::with_max(3))
            .insert(HealthPoints::new(3))
            .insert(CombatConfig {
                attack: 1,
                defense: 11,
            })
            .insert(HexComponent(Hex::new(4, i - 4)));

        commands
            .spawn(SpriteBundle {
                texture: tree_image.clone(),
                transform: Transform::from_xyz(0., 0., ZOrdering::UNITS)
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(UnitMarker)
            .insert(Team::Blue)
            .insert(ActionPoints::with_max(3))
            .insert(HealthPoints::new(3))
            .insert(CombatConfig {
                attack: 1,
                defense: 11,
            })
            .insert(HexComponent(Hex::new(-4, i)));
    }
}
