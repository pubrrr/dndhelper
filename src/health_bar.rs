use bevy::prelude::shape::Quad;
use bevy::prelude::{
    default, info, warn, Added, Assets, BuildChildren, Changed, Children, Color, ColorMaterial,
    ColorMesh2dBundle, Commands, Component, DespawnRecursiveExt, Entity, FromWorld, Handle, Mesh,
    Query, Res, Resource, SpriteBundle, Transform, Vec3, With, Without, World,
};
use bevy::sprite::Mesh2dHandle;

use crate::combat::HealthPoints;
use crate::hex::HEX_RADIUS;
use crate::z_ordering::ZOrdering;

const MAX_Y_SCALE: f32 = 0.9;

#[derive(Resource, Debug)]
pub struct HealthBarResources {
    pub quad_mesh: Mesh2dHandle,
    pub background_color: Handle<ColorMaterial>,
    pub green_color: Handle<ColorMaterial>,
    pub red_color: Handle<ColorMaterial>,
}

#[derive(Component, Debug)]
pub struct HealthBarForEntity {
    entity: Entity,
}

#[derive(Component)]
pub struct HealthIndicatorBarMarker;

impl FromWorld for HealthBarResources {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let quad_mesh = meshes.add(Mesh::from(Quad::default())).into();
        let mut color_materials = world.resource_mut::<Assets<ColorMaterial>>();
        let background_color = color_materials.add(Color::BLACK.into());
        let green_color = color_materials.add(Color::GREEN.into());
        let red_color = color_materials.add(Color::RED.into());

        Self {
            quad_mesh,
            background_color,
            green_color,
            red_color,
        }
    }
}

pub fn add_health_bars(
    mut commands: Commands,
    entities_with_health: Query<Entity, Added<HealthPoints>>,
    health_bar_resources: Res<HealthBarResources>,
) {
    for entity in entities_with_health.iter() {
        info!("adding health bar for {entity:?}");
        commands
            .spawn((
                HealthBarForEntity { entity },
                SpriteBundle {
                    transform: Transform::from_xyz(0., 0., ZOrdering::HEALTH_BAR)
                        .with_scale(Vec3::new(HEX_RADIUS, HEX_RADIUS / 4., 1.)),
                    ..default()
                },
            ))
            .with_children(|child_builder| {
                child_builder.spawn(ColorMesh2dBundle {
                    mesh: health_bar_resources.quad_mesh.clone(),
                    material: health_bar_resources.background_color.clone(),
                    transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(1.)),
                    ..default()
                });
                child_builder.spawn(ColorMesh2dBundle {
                    mesh: health_bar_resources.quad_mesh.clone(),
                    material: health_bar_resources.red_color.clone(),
                    transform: Transform::from_xyz(0., 0., 1.).with_scale(Vec3::new(
                        MAX_Y_SCALE,
                        0.8,
                        1.,
                    )),
                    ..default()
                });
                child_builder.spawn((
                    ColorMesh2dBundle {
                        mesh: health_bar_resources.quad_mesh.clone(),
                        material: health_bar_resources.green_color.clone(),
                        transform: Transform::from_xyz(0., 0., 2.).with_scale(Vec3::new(
                            MAX_Y_SCALE,
                            0.8,
                            1.,
                        )),
                        ..default()
                    },
                    HealthIndicatorBarMarker,
                ));
            });
    }
}

pub fn update_health_bar_positions(
    mut commands: Commands,
    #[allow(clippy::type_complexity)] entities_with_health: Query<
        &Transform,
        (
            With<HealthPoints>,
            Without<HealthBarForEntity>,
            Changed<Transform>,
        ),
    >,
    mut health_bar_entities: Query<(Entity, &HealthBarForEntity, &mut Transform)>,
) {
    for (health_bar_entity, for_entity, mut health_bar_transform) in health_bar_entities.iter_mut()
    {
        let unit_entity = for_entity.entity;
        let unit_transform = match entities_with_health.get(unit_entity) {
            Ok(result) => result,
            Err(_) => {
                info!("Entity {unit_entity:?} for health bar not found, removing it");
                commands.entity(health_bar_entity).despawn_recursive();
                continue;
            }
        };

        let mut transform = unit_transform.translation;
        transform.y -= HEX_RADIUS / 2.;
        transform.z = 100.;
        health_bar_transform.translation = transform;
    }
}

pub fn update_health_bar_size(
    entities_with_health: Query<
        &HealthPoints,
        (Without<HealthBarForEntity>, Changed<HealthPoints>),
    >,
    health_bar_entities: Query<(&HealthBarForEntity, &Children)>,
    mut health_indicator_transforms: Query<&mut Transform, With<HealthIndicatorBarMarker>>,
) {
    if entities_with_health.is_empty() {
        return;
    }
    for (for_entity, children) in &health_bar_entities {
        let Ok(health_points) = entities_with_health.get(for_entity.entity) else {
            continue;
        };

        let Some(health_indicator_child) = children
            .iter()
            .find(|child| health_indicator_transforms.get(**child).is_ok())
        else {
            warn!(
                "no health indicator child found for {:?}",
                for_entity.entity
            );
            continue;
        };

        let mut transform = health_indicator_transforms
            .get_mut(*health_indicator_child)
            .unwrap();

        transform.scale.x =
            MAX_Y_SCALE * health_points.left as f32 / health_points.get_max() as f32;
    }
}
