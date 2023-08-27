use bevy::asset::AssetServer;
use bevy::prelude::{
    default, App, Assets, Camera, Camera2dBundle, Color, ColorMaterial, ColorMesh2dBundle,
    Commands, Component, Entity, GlobalTransform, Handle, Image, Input, Mesh, MouseButton,
    PluginGroup, PostUpdate, Query, Res, ResMut, Resource, SpriteBundle, Startup, Transform,
    Update, Vec2, Vec3, WindowPlugin, With,
};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::PrimaryWindow;
use bevy::DefaultPlugins;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
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
        .init_collection::<ImageAssets>()
        .add_systems(Startup, (setup_camera, setup_hex_grid, setup_units))
        .add_systems(Update, (ui_system, handle_input))
        .add_systems(PostUpdate, update_transform_from_hex)
        .init_resource::<MyResource>()
        .init_resource::<SelectedUnitResource>()
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

    let default_hex_color = materials.add(Color::BLACK.into());
    let highlight_color = materials.add(Color::RED.into());

    Hex::ZERO
        .spiral_range(0..5)
        .map(|hex_coord| hex_layout.hex_to_world_pos(hex_coord))
        .for_each(|world_coord| {
            commands
                .spawn(ColorMesh2dBundle {
                    mesh: mesh.clone().into(),
                    material: default_hex_color.clone(),
                    transform: Transform::from_xyz(world_coord.x, world_coord.y, 0.)
                        .with_scale(Vec3::splat(0.9)),
                    ..default()
                })
                .insert(HexMarker);
        });

    commands.insert_resource(HexResources {
        hex_layout,
        default_hex_color,
        highlight_color,
    });
}

fn setup_units(mut commands: Commands, image_assets: Res<ImageAssets>) {
    commands
        .spawn(SpriteBundle {
            texture: image_assets.manf.clone(),
            transform: Transform::default().with_scale(Vec3::splat(0.5)),
            ..default()
        })
        .insert(UnitMarker)
        .insert(HexComponent(Hex::new(0, 0)));

    commands
        .spawn(SpriteBundle {
            texture: image_assets.tree.clone(),
            transform: Transform::default().with_scale(Vec3::splat(0.5)),
            ..default()
        })
        .insert(UnitMarker)
        .insert(HexComponent(Hex::new(1, 0)));
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

fn update_transform_from_hex(
    mut hex_entities: Query<(&HexComponent, &mut Transform)>,
    hex_resources: Res<HexResources>,
) {
    hex_entities.for_each_mut(|(hex, mut transform)| {
        let wold_pos = hex_resources.hex_layout.hex_to_world_pos(hex.0);
        transform.translation = Vec3::new(wold_pos.x, wold_pos.y, 0.);
    });
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

#[allow(clippy::too_many_arguments)]
fn handle_input(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    hex_resources: Res<HexResources>,
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    windows: Query<&bevy::window::Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut units: Query<(Entity, &mut HexComponent), With<UnitMarker>>,
    hexes: Query<(Entity, &GlobalTransform), With<HexMarker>>,
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

        let Some((clicked_hex_entity, _)) = hexes.iter().find(|(_, global_transform)| {
            hex_resources.hex_layout.world_pos_to_hex(Vec2::new(
                global_transform.translation().x,
                global_transform.translation().y,
            )) == hex_cursor_position
        }) else { return };

        if let Some((entity, _)) = units.iter().find(|(_, hex)| hex.0 == hex_cursor_position) {
            if let Some(selected_unit) = &selected_unit_resource.selected_unit {
                if selected_unit.hex_entity != clicked_hex_entity {
                    commands
                        .entity(selected_unit.hex_entity)
                        .insert(hex_resources.default_hex_color.clone());
                }
            }
            commands
                .entity(clicked_hex_entity)
                .insert(hex_resources.highlight_color.clone());

            selected_unit_resource.selected_unit = Some(SelectedUnit {
                unit_entity: entity,
                hex_entity: clicked_hex_entity,
            });
            return;
        }

        if let Some(selected_unit) = &mut selected_unit_resource.selected_unit {
            if let Ok((_, mut hex)) = units.get_mut(selected_unit.unit_entity) {
                hex.0 = hex_cursor_position;
                commands
                    .entity(clicked_hex_entity)
                    .insert(hex_resources.highlight_color.clone());
                commands
                    .entity(selected_unit.hex_entity)
                    .insert(hex_resources.default_hex_color.clone());
                selected_unit.hex_entity = clicked_hex_entity;
            }
        }
    }
}

#[derive(Resource, Default)]
struct MyResource {
    label: bool,
}

#[derive(Resource)]
struct HexResources {
    hex_layout: HexLayout,
    default_hex_color: Handle<ColorMaterial>,
    highlight_color: Handle<ColorMaterial>,
}

#[derive(Resource, Default)]
struct SelectedUnitResource {
    selected_unit: Option<SelectedUnit>,
}
struct SelectedUnit {
    unit_entity: Entity,
    hex_entity: Entity,
}

#[derive(Component)]
struct HexMarker;

#[derive(Component)]
struct UnitMarker;

#[derive(Component)]
struct HexComponent(Hex);

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "manf.png")]
    manf: Handle<Image>,
    #[asset(path = "tree2.png")]
    tree: Handle<Image>,
}
