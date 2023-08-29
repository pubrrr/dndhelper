use bevy::prelude::{
    Camera, GlobalTransform, Input, MouseButton, Query, Res, ResMut, Resource, With,
};
use bevy::window::PrimaryWindow;
use hexx::Hex;

use crate::common_components::HexFilter;
use crate::hex::{HexComponent, HexResources};

#[derive(Resource, Default)]
pub struct ClickedHex(pub Option<Hex>);

pub fn update_clicked_hex(
    mut clicked_hex: ResMut<ClickedHex>,
    hex_resources: Res<HexResources>,
    buttons: Res<Input<MouseButton>>,
    windows: Query<&bevy::window::Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    hexes: Query<&HexComponent, HexFilter>,
) {
    let window = windows.single();
    let (camera, cam_transform) = cameras.single();

    clicked_hex.0 = None;

    if let Some(hex_cursor_position) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(cam_transform, p))
        .map(|position| hex_resources.hex_layout.world_pos_to_hex(position))
    {
        if !buttons.just_pressed(MouseButton::Left) {
            return;
        }

        if hexes.iter().any(|hex| hex.0 == hex_cursor_position) {
            clicked_hex.0 = Some(hex_cursor_position);
        }
    }
}
