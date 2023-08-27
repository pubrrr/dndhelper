use bevy::prelude::{
    default, Assets, Color, ColorMaterial, ColorMesh2dBundle, Commands, Component, Handle, Mesh,
    ResMut, Resource, Transform, Vec2, Vec3,
};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use hexx::{Hex, HexLayout, PlaneMeshBuilder};

#[derive(Component)]
pub struct HexMarker;

#[derive(Component)]
pub struct HexComponent(pub Hex);

#[derive(Resource)]
pub struct HexResources {
    pub hex_layout: HexLayout,
    pub default_hex_color: Handle<ColorMaterial>,
    pub highlight_color: Handle<ColorMaterial>,
}

pub fn setup_hex_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let hex_layout = HexLayout {
        hex_size: Vec2::splat(50.),
        ..default()
    };

    let mesh = meshes.add(hexagonal_plane(&hex_layout));

    let default_hex_color = materials.add(Color::BLACK.into());
    let highlight_color = materials.add(Color::GREEN.into());

    Hex::ZERO.spiral_range(0..5).for_each(|hex_coord| {
        let world_coord = hex_layout.hex_to_world_pos(hex_coord);
        commands
            .spawn(ColorMesh2dBundle {
                mesh: mesh.clone().into(),
                material: default_hex_color.clone(),
                transform: Transform::from_xyz(world_coord.x, world_coord.y, 0.)
                    .with_scale(Vec3::splat(0.9)),
                ..default()
            })
            .insert(HexComponent(hex_coord))
            .insert(HexMarker);
    });

    commands.insert_resource(HexResources {
        hex_layout,
        default_hex_color,
        highlight_color,
    });
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
