use bevy::prelude::{Camera, Entity, GlobalTransform, Query, Res, ResMut, Resource, With};
use bevy::window::PrimaryWindow;
use hexx::Hex;

use crate::common_components::HexFilter;
use crate::hex::{HexComponent, HexResources};

#[derive(Resource, Default)]
pub struct HoveredUnitResource(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct HoveredHex(pub Option<Hex>);

pub fn update_hovered_hex(
    mut clicked_hex: ResMut<HoveredHex>,
    hex_resources: Res<HexResources>,
    windows: Query<&bevy::window::Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    hexes: Query<&HexComponent, HexFilter>,
) {
    let window = windows.single();
    let (camera, cam_transform) = cameras.single();

    if let Some(hex_cursor_position) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(cam_transform, p))
        .map(|position| hex_resources.hex_layout.world_pos_to_hex(position))
    {
        if hexes.iter().any(|hex| hex.0 == hex_cursor_position) {
            clicked_hex.0 = Some(hex_cursor_position);
            return;
        }
    }
    clicked_hex.0 = None;
}
