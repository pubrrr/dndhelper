use crate::selected_unit::SelectedUnitHexMarker;
use bevy::prelude::{
    default, Assets, Color, ColorMaterial, ColorMesh2dBundle, Commands, Component, Handle, Mesh,
    ResMut, Resource, Transform, Vec2, Vec3,
};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::utils::HashMap;
use hexx::{Hex, HexLayout, PlaneMeshBuilder};

use crate::terrain::MovementCost;
use crate::terrain::Terrain;
use crate::z_ordering::ZOrdering;

pub const HEX_RADIUS: f32 = 50.;

#[derive(Component)]
pub struct HexMarker;

#[derive(Component)]
pub struct HexOverlayMarker;

#[derive(Component)]
pub struct HexComponent(pub Hex);

#[derive(Resource)]
pub struct HexResources {
    pub hex_layout: HexLayout,
    pub not_reachable_overlay_color: Handle<ColorMaterial>,
}

pub fn setup_hex_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let hex_layout = HexLayout {
        hex_size: Vec2::splat(HEX_RADIUS),
        ..default()
    };

    let mesh = meshes.add(hexagonal_plane(&hex_layout));

    let not_reachable_overlay_color = materials.add(Color::DARK_GRAY.with_a(0.5).into());

    let terrain_map = build_terrain_map(&mut materials);

    Hex::ZERO
        .spiral_range(0..5)
        .enumerate()
        .for_each(|(i, hex_coord)| {
            let terrain_key = match i % 4 {
                3 => "forest",
                2 => "water",
                _ => "plains",
            };
            let (terrain, color) = &terrain_map[terrain_key];

            let world_coord = hex_layout.hex_to_world_pos(hex_coord);
            commands
                .spawn(ColorMesh2dBundle {
                    mesh: mesh.clone().into(),
                    material: color.clone(),
                    transform: Transform::from_xyz(world_coord.x, world_coord.y, ZOrdering::HEX)
                        .with_scale(Vec3::splat(0.9)),
                    ..default()
                })
                .insert(HexComponent(hex_coord))
                .insert(terrain.clone())
                .insert(HexMarker);
            commands
                .spawn(HexOverlayMarker)
                .insert(ColorMesh2dBundle {
                    mesh: mesh.clone().into(),
                    material: not_reachable_overlay_color.clone(),
                    transform: Transform::from_xyz(
                        world_coord.x,
                        world_coord.y,
                        ZOrdering::HEX_OVERLAY,
                    )
                    .with_scale(Vec3::splat(0.9)),
                    ..default()
                })
                .insert(HexComponent(hex_coord));
        });

    commands
        .spawn(ColorMesh2dBundle {
            mesh: mesh.clone().into(),
            transform: Transform::from_xyz(0., 0., ZOrdering::SELECTED_UNIT_HEX)
                .with_scale(Vec3::splat(1.)),
            ..default()
        })
        .insert(HexComponent(Hex::ZERO))
        .insert(SelectedUnitHexMarker {
            selected_hex_color: materials.add(Color::YELLOW.into()),
        });

    commands.insert_resource(HexResources {
        hex_layout,
        not_reachable_overlay_color,
    });
}

fn build_terrain_map<'a>(
    materials: &mut Assets<ColorMaterial>,
) -> HashMap<&'a str, (Terrain, Handle<ColorMaterial>)> {
    HashMap::from([
        (
            "plains",
            (
                Terrain {
                    name: "Plains".to_string(),
                    movement_cost: MovementCost::Passable(1),
                },
                materials.add(Color::YELLOW_GREEN.into()),
            ),
        ),
        (
            "forest",
            (
                Terrain {
                    name: "Forest".to_string(),
                    movement_cost: MovementCost::Passable(2),
                },
                materials.add(Color::DARK_GREEN.into()),
            ),
        ),
        (
            "water",
            (
                Terrain {
                    name: "Water".to_string(),
                    movement_cost: MovementCost::Impassable,
                },
                materials.add(Color::BLUE.into()),
            ),
        ),
    ])
}

fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(hex_layout).facing(Vec3::Z).build();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs);
    mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
    mesh
}
