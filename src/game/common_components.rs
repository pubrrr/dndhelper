use bevy::prelude::{Component, With, Without};

use crate::game::hex::HexMarker;

pub type UnitFilter = (With<UnitMarker>, Without<HexMarker>);
pub type HexFilter = (With<HexMarker>, Without<UnitMarker>);

#[derive(Component)]
pub struct UnitMarker(pub String);
