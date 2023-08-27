use bevy::asset::AssetServer;
use bevy::prelude::{
    default, App, Assets, Camera, Camera2dBundle, Color, ColorMaterial, ColorMesh2dBundle,
    Commands, Component, Entity, GlobalTransform, Handle, Image, Input, Mesh, MouseButton,
    PluginGroup, PostStartup, PostUpdate, Query, Res, ResMut, Resource, SpriteBundle, Startup,
    Transform, Update, Vec2, Vec3, WindowPlugin, With, Without,
};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use bevy::DefaultPlugins;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use bevy_egui::egui::Window;
use bevy_egui::{EguiContexts, EguiPlugin};
use hexx::{Hex, HexLayout, PlaneMeshBuilder};

type UnitFilter = (With<UnitMarker>, Without<HexMarker>);
type HexFilter = (With<HexMarker>, Without<UnitMarker>);

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
        .add_systems(
            Startup,
            (setup_camera, setup_hex_grid, setup_team_resources),
        )
        .add_systems(PostStartup, setup_teams)
        .add_systems(Update, (ui_system, handle_input))
        .add_systems(PostUpdate, (update_transform_from_hex, update_hex_colors))
        .init_resource::<MyResource>()
        .init_resource::<SelectedUnitResource>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_team_resources(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let red_hex_color = materials.add(Color::RED.into());
    let blue_hex_color = materials.add(Color::BLUE.into());

    commands.insert_resource(TeamResources {
        materials: [
            (
                Team::Red,
                TeamMaterial {
                    hex_color: red_hex_color,
                },
            ),
            (
                Team::Blue,
                TeamMaterial {
                    hex_color: blue_hex_color,
                },
            ),
        ]
        .iter()
        .cloned()
        .collect(),
    });
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

fn setup_teams(mut commands: Commands, image_assets: Res<ImageAssets>) {
    for i in 0..5 {
        commands
            .spawn(SpriteBundle {
                texture: image_assets.manf.clone(),
                transform: Transform::default().with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(UnitMarker)
            .insert(Team::Red)
            .insert(HexComponent(Hex::new(4, i - 4)));

        commands
            .spawn(SpriteBundle {
                texture: image_assets.tree.clone(),
                transform: Transform::default().with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(UnitMarker)
            .insert(Team::Blue)
            .insert(HexComponent(Hex::new(-4, i)));
    }
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
    mut hex_entities: Query<(&HexComponent, &mut Transform), Without<HexMarker>>,
    hex_resources: Res<HexResources>,
) {
    hex_entities.for_each_mut(|(hex, mut transform)| {
        let wold_pos = hex_resources.hex_layout.hex_to_world_pos(hex.0);
        transform.translation = Vec3::new(wold_pos.x, wold_pos.y, 0.);
    });
}

fn update_hex_colors(
    units: Query<(Entity, &HexComponent, &Team), UnitFilter>,
    mut hex_entities: Query<(&HexComponent, &mut Handle<ColorMaterial>), HexFilter>,
    team_resources: Res<TeamResources>,
    hex_resources: Res<HexResources>,
    selected_unit_resource: Res<SelectedUnitResource>,
) {
    let mut hex_to_color_map = HashMap::from_iter(
        units
            .iter()
            .map(|(_, hex, team)| (hex.0, team_resources.materials[team].hex_color.clone())),
    );

    if let Some(selected_unit) = &selected_unit_resource.selected_unit {
        let selected_unit_hex = units
            .get_component::<HexComponent>(*selected_unit)
            .unwrap()
            .0;
        hex_to_color_map.insert(selected_unit_hex, hex_resources.highlight_color.clone());
    }

    hex_entities.for_each_mut(|(hex, mut material)| {
        let color = hex_to_color_map
            .get(&hex.0)
            .cloned()
            .unwrap_or_else(|| hex_resources.default_hex_color.clone());
        *material = color;
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

fn handle_input(
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
    selected_unit: Option<Entity>,
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

#[derive(Resource)]
struct TeamResources {
    materials: HashMap<Team, TeamMaterial>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
enum Team {
    Red,
    Blue,
}

#[derive(Clone)]
struct TeamMaterial {
    hex_color: Handle<ColorMaterial>,
}
