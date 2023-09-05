use std::collections::HashSet;

use bevy::prelude::{
    debug, Changed, ColorMaterial, Commands, Component, DetectChanges, Entity, Handle, Input,
    KeyCode, Query, Res, ResMut, Resource, With, Without,
};
use bevy::utils::HashMap;
use hexx::algorithms::field_of_movement;
use hexx::Hex;

use crate::game::action_points::ActionPoints;
use crate::game::common_components::{UnitFilter, UnitMarker};
use crate::game::hex::{HexComponent, HexMarker, HexOverlayMarker, HexResources};
use crate::game::team_setup::Team;
use crate::game::terrain::{MovementCost, Terrain};

#[derive(Resource, Default)]
pub struct SelectedUnitResource {
    selected_unit: Option<Entity>,
    recompute_cache: bool,
    reachable_hexes: Option<HashSet<Hex>>,
    cost_map: HashMap<Hex, MovementCost>,
}

impl SelectedUnitResource {
    pub fn selected_unit(&self) -> Option<Entity> {
        self.selected_unit
    }

    pub fn set_selected_unit(&mut self, selected_unit: Option<Entity>) {
        self.needs_reachable_hexes_recomputation();
        self.selected_unit = selected_unit;
    }

    pub fn reachable_hexes(&self) -> &Option<HashSet<Hex>> {
        &self.reachable_hexes
    }

    pub fn cost_map(&self) -> &HashMap<Hex, MovementCost> {
        &self.cost_map
    }

    pub fn needs_reachable_hexes_recomputation(&mut self) {
        self.recompute_cache = true;
        self.reachable_hexes = Some(HashSet::new());
        self.cost_map = HashMap::new();
    }
}

#[derive(Component, Clone)]
pub struct SelectedUnitHexMarker {
    pub selected_hex_color: Handle<ColorMaterial>,
}

pub fn check_whether_selected_unit_needs_recomputation(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    units_with_changed_action_points: Query<Entity, (With<UnitMarker>, Changed<ActionPoints>)>,
) {
    let Some(selected_unit) = selected_unit_resource.selected_unit else {
        return;
    };

    if units_with_changed_action_points.contains(selected_unit) {
        debug!("Recomputing because selected unit action points changed");
        selected_unit_resource.needs_reachable_hexes_recomputation();
    }
}

pub fn reset_selected_unit(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        selected_unit_resource.set_selected_unit(None);
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
    if !selected_unit_resource.is_changed() {
        return;
    }

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

pub fn update_reachable_hexes_cache(
    units: Query<(&ActionPoints, &HexComponent, &Team, &ActionPoints), UnitFilter>,
    hexes: Query<(&HexComponent, &Terrain), With<HexMarker>>,
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
) {
    if !selected_unit_resource.recompute_cache {
        return;
    }

    let Some(selected_unit) = selected_unit_resource.selected_unit else {
        selected_unit_resource.reachable_hexes = None;
        selected_unit_resource.recompute_cache = false;
        return;
    };

    let (action_points, selected_unit_hex, selected_unit_team, selected_unit_action_points) =
        units.get(selected_unit).unwrap();

    selected_unit_resource.cost_map = hexes
        .iter()
        .map(|(hex_component, terrain)| (hex_component.0, terrain.movement_cost.clone()))
        .collect();

    selected_unit_resource
        .cost_map
        .extend(units.iter().map(|(_, hex_component, _, _)| {
            let cost = match selected_unit_hex.0.unsigned_distance_to(hex_component.0) {
                0 => MovementCost::Passable(0),
                _ => MovementCost::Impassable,
            };
            (hex_component.0, cost)
        }));

    let mut reachable_hexes =
        field_of_movement(selected_unit_hex.0, action_points.left as u32, |hex| {
            selected_unit_resource
                .cost_map
                .get(&hex)
                .and_then(|movement_cost| movement_cost.get_modified_algorithm_cost())
        });

    if selected_unit_action_points.can_still_attack_this_turn() {
        let attackable_units = units
            .iter()
            .filter(|(_, _, team, _)| team != &selected_unit_team)
            .filter(|(_, hex_component, _, _)| {
                selected_unit_hex.0.unsigned_distance_to(hex_component.0) == 1
            })
            .map(|(_, hex_component, _, _)| hex_component.0);

        reachable_hexes.extend(attackable_units);
    }

    selected_unit_resource.reachable_hexes = Some(reachable_hexes);
    selected_unit_resource.recompute_cache = false;
}

pub fn update_hex_overlay(
    mut commands: Commands,
    hex_overlays: Query<
        (Entity, &HexComponent, Option<&Handle<ColorMaterial>>),
        With<HexOverlayMarker>,
    >,
    hex_resources: Res<HexResources>,
    selected_unit_resource: Res<SelectedUnitResource>,
) {
    if let Some(reachable_hexes) = &selected_unit_resource.reachable_hexes {
        for (entity, hex, color_material) in &hex_overlays {
            let is_reachable = reachable_hexes.contains(&hex.0);
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
