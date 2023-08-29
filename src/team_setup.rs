use crate::combat::AttackConfig;
use crate::combat::HealthPoints;
use bevy::prelude::{
    default, Assets, Color, ColorMaterial, Commands, Component, Handle, Res, ResMut, Resource,
    SpriteBundle, Transform, Vec3,
};
use bevy::utils::HashMap;
use hexx::Hex;
use std::fmt::{Display, Formatter};

use crate::action_points::ActionPoints;
use crate::common_components::UnitMarker;
use crate::hex::HexComponent;
use crate::ImageAssets;

#[derive(Resource)]
pub struct TeamResources {
    pub materials: HashMap<Team, TeamMaterial>,
}

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

#[derive(Clone)]
pub struct TeamMaterial {
    pub hex_color: Handle<ColorMaterial>,
}

pub fn setup_team_resources(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let red_hex_color = materials.add(Color::RED.into());
    let blue_hex_color = materials.add(Color::BLUE.into());

    commands.insert_resource(TeamResources {
        materials: [
            (
                Team::Red,
                TeamMaterial {
                    hex_color: red_hex_color,
                },
            ),
            (
                Team::Blue,
                TeamMaterial {
                    hex_color: blue_hex_color,
                },
            ),
        ]
        .iter()
        .cloned()
        .collect(),
    });
}

pub fn setup_team_units(mut commands: Commands, image_assets: Res<ImageAssets>) {
    for i in 0..5 {
        commands
            .spawn(SpriteBundle {
                texture: image_assets.manf.clone(),
                transform: Transform::default().with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(UnitMarker)
            .insert(Team::Red)
            .insert(ActionPoints::with_max(3))
            .insert(HealthPoints::new(3))
            .insert(AttackConfig { attack: 1 })
            .insert(HexComponent(Hex::new(4, i - 4)));

        commands
            .spawn(SpriteBundle {
                texture: image_assets.tree.clone(),
                transform: Transform::default().with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(UnitMarker)
            .insert(Team::Blue)
            .insert(ActionPoints::with_max(3))
            .insert(HealthPoints::new(3))
            .insert(AttackConfig { attack: 1 })
            .insert(HexComponent(Hex::new(-4, i)));
    }
}
