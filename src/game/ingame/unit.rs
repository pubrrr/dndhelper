use crate::game::ingame::action_points::ActionPoints;
use crate::game::ingame::combat::{CombatConfig, HealthPoints};
use bevy::prelude::{Bundle, Component, Handle, Image, SpriteBundle, Transform, With, Without};
use hexx::Hex;

use crate::game::ingame::hex::{HexComponent, HexMarker};
use crate::game::ingame::team_setup::Team;
use crate::game::ingame::unit_status::UnitStatus;

pub type UnitFilter = (With<UnitMarker>, Without<HexMarker>);
pub type HexFilter = (With<HexMarker>, Without<UnitMarker>);

#[derive(Component, Debug, Eq, PartialEq)]
pub struct UnitMarker(pub String);

#[derive(Bundle)]
pub struct UnitBundle {
    sprite_bundle: SpriteBundle,
    unit_marker: UnitMarker,
    player: Team,
    unit_status: UnitStatus,
    action_points: ActionPoints,
    health_points: HealthPoints,
    combat_config: CombatConfig,
    hex_component: HexComponent,
}

pub struct ProtoUnitBundle {
    pub texture: Handle<Image>,
    pub transform: Transform,
    pub unit_marker: UnitMarker,
    pub player: Team,
    pub action_points: ActionPoints,
    pub health_points: HealthPoints,
    pub combat_config: CombatConfig,
    pub hex: Hex,
}

impl From<ProtoUnitBundle> for UnitBundle {
    fn from(value: ProtoUnitBundle) -> Self {
        let ProtoUnitBundle {
            texture,
            transform,
            unit_marker,
            player,
            action_points,
            health_points,
            combat_config,
            hex,
        } = value;

        Self {
            sprite_bundle: SpriteBundle {
                texture,
                transform,
                ..Default::default()
            },
            unit_marker,
            player,
            unit_status: UnitStatus::new(),
            action_points,
            health_points,
            combat_config,
            hex_component: HexComponent(hex),
        }
    }
}
