use bevy::prelude::{ColorMaterial, Handle, Query, Res, Transform, Vec3, Without};
use bevy::utils::HashMap;

use crate::action_points::ActionPoints;
use crate::common_components::{HexFilter, UnitFilter};
use crate::hex::{HexComponent, HexMarker, HexResources};
use crate::team_setup::{Team, TeamResources};
use crate::SelectedUnitResource;

pub fn update_transform_from_hex(
    mut hex_entities: Query<(&HexComponent, &mut Transform), Without<HexMarker>>,
    hex_resources: Res<HexResources>,
) {
    hex_entities.for_each_mut(|(hex, mut transform)| {
        let wold_pos = hex_resources.hex_layout.hex_to_world_pos(hex.0);
        transform.translation = Vec3::new(wold_pos.x, wold_pos.y, 0.);
    });
}

pub fn update_hex_colors(
    units: Query<(&ActionPoints, &HexComponent, &Team), UnitFilter>,
    mut hex_entities: Query<(&HexComponent, &mut Handle<ColorMaterial>), HexFilter>,
    team_resources: Res<TeamResources>,
    hex_resources: Res<HexResources>,
    selected_unit_resource: Res<SelectedUnitResource>,
) {
    let mut hex_to_color_map = HashMap::from_iter(
        units
            .iter()
            .map(|(_, hex, team)| (hex.0, &team_resources.materials[team].hex_color)),
    );

    let mut default_color = hex_resources.default_hex_color.clone();

    if let Some(selected_unit) = &selected_unit_resource.selected_unit {
        default_color = hex_resources.not_reachable_hex_color.clone();

        let selected_unit_hex = units
            .get_component::<HexComponent>(*selected_unit)
            .unwrap()
            .0;
        hex_to_color_map.insert(selected_unit_hex, &hex_resources.highlight_color);

        let action_points = units.get_component::<ActionPoints>(*selected_unit).unwrap();
        let reachable_hexes = hex_entities
            .iter()
            .filter(|(hex, _)| {
                selected_unit_hex.unsigned_distance_to(hex.0) as usize <= action_points.left
            })
            .map(|(hex, _)| hex.0);
        for reachable_hex in reachable_hexes {
            if !hex_to_color_map.contains_key(&reachable_hex) {
                hex_to_color_map.insert(reachable_hex, &hex_resources.default_hex_color);
            }
        }
    }

    hex_entities.for_each_mut(|(hex, mut material)| {
        let color = hex_to_color_map
            .get(&hex.0)
            .map(|color| (*color).clone())
            .unwrap_or_else(|| default_color.clone());
        *material = color;
    });
}
