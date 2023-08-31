use crate::action_points::ActionPoints;
use crate::common_components::{UnitFilter, UnitMarker};
use crate::hex::{HexComponent, HexOverlayMarker, HexResources};
use bevy::prelude::{
    ColorMaterial, Commands, Component, Entity, Handle, Input, KeyCode, Query, Res, ResMut,
    Resource, With, Without,
};

#[derive(Resource, Default)]
pub struct SelectedUnitResource {
    pub selected_unit: Option<Entity>,
}

#[derive(Component, Clone)]
pub struct SelectedUnitHexMarker {
    pub selected_hex_color: Handle<ColorMaterial>,
}

pub fn reset_selected_unit(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        selected_unit_resource.selected_unit = None;
    }
}

pub fn update_selected_unit_hex(
    mut commands: Commands,
    selected_unit_resource: Res<SelectedUnitResource>,
    units: Query<&HexComponent, UnitFilter>,
    mut selected_unit_hex_query: Query<
        (
            Entity,
            &mut HexComponent,
            &SelectedUnitHexMarker,
            Option<&Handle<ColorMaterial>>,
        ),
        Without<UnitMarker>,
    >,
) {
    if let Some(selected_unit) = selected_unit_resource.selected_unit {
        let selected_unit_hex_component = units.get(selected_unit).unwrap();
        let (entity, mut hex_hex_component, marker, _) = selected_unit_hex_query.single_mut();

        hex_hex_component.0 = selected_unit_hex_component.0;
        commands
            .entity(entity)
            .insert(marker.selected_hex_color.clone());

        return;
    }
    let (entity, _, _, color_material) = selected_unit_hex_query.single();

    if color_material.is_some() {
        commands.entity(entity).remove::<Handle<ColorMaterial>>();
    }
}

pub fn update_hex_colors(
    mut commands: Commands,
    units: Query<(&ActionPoints, &HexComponent), UnitFilter>,
    hex_overlays: Query<
        (Entity, &HexComponent, Option<&Handle<ColorMaterial>>),
        With<HexOverlayMarker>,
    >,
    hex_resources: Res<HexResources>,
    selected_unit_resource: Res<SelectedUnitResource>,
) {
    if let Some(selected_unit) = selected_unit_resource.selected_unit {
        let (action_points, selected_unit_hex) = units.get(selected_unit).unwrap();

        for (entity, hex, color_material) in &hex_overlays {
            let is_reachable =
                selected_unit_hex.0.unsigned_distance_to(hex.0) as usize <= action_points.left;
            if color_material.is_some() && is_reachable {
                commands.entity(entity).remove::<Handle<ColorMaterial>>();
            } else if color_material.is_none() && !is_reachable {
                commands
                    .entity(entity)
                    .insert(hex_resources.not_reachable_overlay_color.clone());
            }
        }

        return;
    }

    for (entity, _, color_material) in &hex_overlays {
        if color_material.is_some() {
            commands.entity(entity).remove::<Handle<ColorMaterial>>();
        }
    }
}
