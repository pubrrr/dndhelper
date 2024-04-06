use bevy::prelude::{
    default, trace, Color, Commands, Component, DespawnRecursiveExt, DetectChanges, Entity, Query,
    Res, ResMut, Resource, SpatialBundle, Transform, With,
};
use bevy_prototype_lyon::prelude::{Fill, PathBuilder, ShapeBundle, Stroke};
use hexx::algorithms::a_star;
use hexx::Hex;

use crate::game::ingame::hex::{HexComponent, HexResources};
use crate::game::ingame::hovered_hex::HoveredHex;
use crate::game::ingame::selected_unit::SelectedUnitResource;
use crate::game::ingame::unit::UnitMarker;
use crate::game::ingame::z_ordering::ZOrdering;

#[derive(Resource, Default, Debug)]
pub struct CurrentPath(pub Option<Vec<Hex>>);

#[derive(Component, Debug)]
pub struct PathMarker;

pub(super) fn despawn_old_path(
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

pub(super) fn compute_current_path(
    mut commands: Commands,
    selected_unit_resource: Res<SelectedUnitResource>,
    hovered_hex: Res<HoveredHex>,
    hex_resources: Res<HexResources>,
    units: Query<&HexComponent, With<UnitMarker>>,
    mut current_path: ResMut<CurrentPath>,
) {
    if !selected_unit_resource.is_changed() && !hovered_hex.is_changed() {
        return;
    };

    let Some(selected_unit) = selected_unit_resource.selected_unit() else {
        current_path.0 = None;
        return;
    };
    let Some(hovered_hex) = hovered_hex.0 else {
        current_path.0 = None;
        return;
    };
    let Ok(unit_hex) = units.get(selected_unit) else {
        current_path.0 = None;
        return;
    };

    let Some(hexes_way) = a_star(unit_hex.0, hovered_hex, |hex, _| {
        selected_unit_resource
            .cost_map()
            .get(&hex)
            .and_then(|movement_cost| movement_cost.get_modified_algorithm_cost())
    }) else {
        current_path.0 = None;
        return;
    };

    let world_pos_way = hexes_way
        .iter()
        .map(|hex| hex_resources.hex_layout.hex_to_world_pos(*hex));

    let path_segments = world_pos_way.clone().zip(world_pos_way.skip(1));

    for (from, to) in path_segments {
        trace!("Spawning path line from {from} to {to}");

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(from);
        path_builder.line_to(to);

        commands
            .spawn(PathMarker)
            .insert(ShapeBundle {
                path: path_builder.build(),
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(0., 0., ZOrdering::PATH_LINES),
                    ..default()
                },
                ..default()
            })
            .insert(Stroke::new(Color::BLACK, 10.))
            .insert(Fill::color(Color::RED));
    }

    current_path.0 = Some(hexes_way);
}
