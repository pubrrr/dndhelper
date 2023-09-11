use crate::game::common_components::UnitMarker;
use crate::game::hex::{HexComponent, HexResources};
use bevy::prelude::{
    debug, default, info, trace, warn, Color, Commands, Component, DespawnRecursiveExt,
    DetectChanges, Entity, Query, Res, Transform, With,
};
use bevy_prototype_lyon::prelude::{Fill, PathBuilder, ShapeBundle, Stroke};
use hexx::algorithms::a_star;
use hexx::Hex;

use crate::game::hovered_hex::HoveredHex;
use crate::game::selected_unit::SelectedUnitResource;
use crate::game::z_ordering::ZOrdering;

#[derive(Debug)]
pub struct CurrentPath(Option<Vec<Hex>>);

#[derive(Component, Debug)]
pub struct PathMarker;

pub fn despawn_old_path(
    mut commands: Commands,
    selected_unit_resource: Res<SelectedUnitResource>,
    hovered_hex: Res<HoveredHex>,
    path_entities: Query<Entity, With<PathMarker>>,
) {
    if !(selected_unit_resource.is_changed() || hovered_hex.is_changed()) {
        return;
    };

    for path_entity in &path_entities {
        trace!("despawning path entity {path_entity:?}");
        commands.entity(path_entity).despawn_recursive();
    }
}

pub fn compute_current_path(
    mut commands: Commands,
    selected_unit_resource: Res<SelectedUnitResource>,
    hovered_hex: Res<HoveredHex>,
    hex_resources: Res<HexResources>,
    units: Query<&HexComponent, With<UnitMarker>>,
) {
    if !selected_unit_resource.is_changed() && !hovered_hex.is_changed() {
        return;
    };

    let Some(selected_unit) = selected_unit_resource.selected_unit() else {
        return;
    };
    let Some(hovered_hex) = hovered_hex.0 else {
        return;
    };
    let Ok(unit_hex) = units.get(selected_unit) else {
        return;
    };

    let Some(hexes_way) = a_star(unit_hex.0, hovered_hex, |hex| {
        selected_unit_resource
            .cost_map()
            .get(&hex)
            .and_then(|movement_cost| movement_cost.get_modified_algorithm_cost())
    }) else {
        return;
    };

    let world_pos_way = hexes_way
        .into_iter()
        .map(|hex| hex_resources.hex_layout.hex_to_world_pos(hex));

    let path_segments = world_pos_way.clone().zip(world_pos_way.skip(1));

    for (from, to) in path_segments {
        trace!("Spawning path line from {from} to {to}");

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(from);
        path_builder.line_to(to);

        commands.spawn((
            PathMarker,
            ShapeBundle {
                path: path_builder.build(),
                transform: Transform::from_xyz(0., 0., ZOrdering::PATH_LINES),
                ..default()
            },
            Stroke::new(Color::BLACK, 10.),
            Fill::color(Color::RED),
        ));
    }
}
