use bevy::prelude::{Query, Res, Transform, Without};

use crate::game::hex::{HexComponent, HexMarker, HexResources};

pub fn update_transform_from_hex(
    mut hex_entities: Query<(&HexComponent, &mut Transform), Without<HexMarker>>,
    hex_resources: Res<HexResources>,
) {
    hex_entities.for_each_mut(|(hex, mut transform)| {
        let wold_pos = hex_resources.hex_layout.hex_to_world_pos(hex.0);
        transform.translation.x = wold_pos.x;
        transform.translation.y = wold_pos.y;
    });
}
