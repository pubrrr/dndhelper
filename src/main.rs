use bevy::prelude::{
    default, info, App, Assets, Camera, Camera2dBundle, Color, ColorMaterial, ColorMesh2dBundle,
    Commands, Component, Entity, GlobalTransform, Handle, Input, Mesh, MouseButton, PluginGroup,
    Query, Res, ResMut, Resource, Startup, Transform, Update, Vec2, Vec3, WindowPlugin, With,
};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::PrimaryWindow;
use bevy::DefaultPlugins;
use bevy_egui::egui::Window;
use bevy_egui::{EguiContexts, EguiPlugin};
use hexx::{Hex, HexLayout, PlaneMeshBuilder};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(bevy::window::Window {
                    resolution: (1000., 1000.).into(),
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin,
        ))
        .add_systems(Startup, (setup_camera, setup_hex_grid))
        .add_systems(Update, (ui_system, handle_input))
        .init_resource::<MyResource>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_hex_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let hex_layout = HexLayout {
        hex_size: Vec2::splat(50.),
        ..default()
    };

    let mesh = meshes.add(hexagonal_plane(&hex_layout));

    let cyan = materials.add(Color::CYAN.into());
    let red = materials.add(Color::SALMON.into());

    Hex::ZERO
        .spiral_range(0..5)
        .map(|hex_coord| {
            let vec2 = hex_layout.hex_to_world_pos(hex_coord);
            info!("hex_coord: {hex_coord:?}, world_coord: {vec2}");

            vec2
        })
        .for_each(|world_coord| {
            commands.spawn((
                ColorMesh2dBundle {
                    mesh: mesh.clone().into(),
                    material: cyan.clone(),
                    transform: Transform::from_xyz(world_coord.x, world_coord.y, 0.)
                        .with_scale(Vec3::splat(0.9)),
                    ..default()
                },
                HexMarker,
            ));
        });

    commands.insert_resource(HexResources { red, hex_layout });
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

fn ui_system(mut contexts: EguiContexts, mut resource: ResMut<MyResource>) {
    Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label(match resource.label {
            true => "hi",
            false => "asdfasdf",
        });
        if ui.button("Click").clicked() {
            resource.label = !resource.label
        };
    });
}

fn handle_input(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    hex_resources: Res<HexResources>,
    windows: Query<&bevy::window::Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    hexes: Query<(Entity, &GlobalTransform), With<HexMarker>>,
) {
    let window = windows.single();
    let (camera, cam_transform) = cameras.single();
    if let Some(hex_cursor_position) = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(cam_transform, p))
        .map(|position| hex_resources.hex_layout.world_pos_to_hex(position))
    {
        if buttons.just_pressed(MouseButton::Left) {
            if let Some((entity, _)) = hexes.iter().find(|(_, global_transform)| {
                hex_resources.hex_layout.world_pos_to_hex(Vec2::new(
                    global_transform.translation().x,
                    global_transform.translation().y,
                )) == hex_cursor_position
            }) {
                commands.entity(entity).insert(hex_resources.red.clone());
            };
        }
    }
}

#[derive(Resource, Default)]
struct MyResource {
    label: bool,
}

#[derive(Resource)]
struct HexResources {
    red: Handle<ColorMaterial>,
    hex_layout: HexLayout,
}

#[derive(Component)]
struct HexMarker;
