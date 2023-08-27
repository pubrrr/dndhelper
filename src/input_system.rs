use bevy::prelude::{
    Camera, Entity, GlobalTransform, Input, MouseButton, Query, Res, ResMut, With,
};
use bevy::window::PrimaryWindow;

use crate::common_components::UnitFilter;
use crate::hex::{HexComponent, HexResources};
use crate::SelectedUnitResource;

pub fn handle_input(
    buttons: Res<Input<MouseButton>>,
    hex_resources: Res<HexResources>,
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    windows: Query<&bevy::window::Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut units: Query<(Entity, &mut HexComponent), UnitFilter>,
) {
    let window = windows.single();
    let (camera, cam_transform) = cameras.single();
    if let Some(hex_cursor_position) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(cam_transform, p))
        .map(|position| hex_resources.hex_layout.world_pos_to_hex(position))
    {
        if !buttons.just_pressed(MouseButton::Left) {
            return;
        }

        if let Some((entity, _)) = units.iter().find(|(_, hex)| hex.0 == hex_cursor_position) {
            selected_unit_resource.selected_unit = Some(entity);
            return;
        }

        if let Some(selected_unit) = selected_unit_resource.selected_unit {
            let (_, mut hex) = units.get_mut(selected_unit).unwrap();
            hex.0 = hex_cursor_position;
        }
    }
}
