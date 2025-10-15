use bevy::asset::Handle;
use bevy::ecs::hierarchy::ChildOf;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::Mesh3d;
use bevy::pbr::MeshMaterial3d;
use bevy::picking::Pickable;
use bevy::prelude::{AlphaMode, Meshable, Quat, Sphere};
use bevy::prelude::{
    Assets, Children, Color, Commands, Component, Entity, GlobalTransform, Mesh, Name, Query,
    ResMut, Resource, StandardMaterial, Transform, Vec3, Visibility, With, Without,
};
use bevy_granite_core::TreeHiddenEntity;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use crate::gizmos::{GizmoConfig, GizmoMode, GizmoOf, GizmoRoot};
use crate::{gizmos::GizmoMesh, input::GizmoAxis};

#[derive(Component)]
pub struct RotateGizmo;

#[derive(Resource, Default, Component)]
pub struct RotateGizmoParent;

#[derive(Resource, Default)]
pub struct PreviousTransformGizmo {
    pub entity: Option<Entity>,
}

const GIZMO_SCALE: f32 = 0.85;
const ROTATE_INNER_RADIUS: f32 = 0.12 * GIZMO_SCALE; // middle sphere of gizmo (free rotate)
const ROTATE_VISUAL_RADIUS: f32 = 0.64 * GIZMO_SCALE; // middle sphere of gizmo (visual)
const RING_MESH_HASH: uuid::Uuid = uuid::uuid!("3f6f4c2a-6e36-4ccf-81c4-f343f83c5f80"); // constantly random - doesnt matter the value

pub fn register_embedded_rotate_gizmo_mesh(mut meshes: ResMut<Assets<Mesh>>) {
    let handle = get_mesh_handle();
    let ring_obj = include_str!("./Ring.obj");
    let ring_mesh =
        bevy_obj::mesh::load_obj_as_mesh(ring_obj.as_bytes(), &bevy_obj::ObjSettings::default())
            .expect("Obj to load");
    meshes.insert(handle.id(), ring_mesh);
}

pub fn get_mesh_handle() -> Handle<Mesh> {
    Handle::Uuid(RING_MESH_HASH, Default::default())
}
pub fn spawn_rotate_gizmo(
    parent: Entity,
    query: &mut Query<&GlobalTransform, Without<RotateGizmoParent>>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    config: GizmoConfig,
) {
    let offset = Vec3::new(0., 0., 0.);

    if let Ok(parent_global_transform) = query.get(parent) {
        let gizmo_translation = offset;

        let initial_rotation = match config.mode() {
            GizmoMode::Global => {
                parent_global_transform
                    .to_scale_rotation_translation()
                    .1
                    .inverse()
            }
            GizmoMode::Local => {
                Quat::IDENTITY
            }
        };

        // Create the gizmo parent entity 
        let gizmo_entity = commands
            .spawn((
                Transform {
                    translation: gizmo_translation,
                    rotation: initial_rotation,
                    ..Default::default()
                },
                Visibility::default(),
                GizmoOf(parent),
                ChildOf(parent),
                config,
            ))
            .insert(Name::new("RotateGizmo"))
            .insert(RotateGizmoParent)
            .insert(TreeHiddenEntity)
            .insert(RotateGizmo)
            .id();

        // Build the visual sphere as a child
        build_visual_sphere(parent, commands, materials, gizmo_entity, meshes);

        build_free_sphere(
            parent,
            commands,
            materials,
            gizmo_entity,
            Color::srgba(1., 1., 0.0, 1.),
            meshes,
        );

        build_axis_ring(
            parent,
            commands,
            materials,
            gizmo_entity,
            GizmoAxis::X,
            Color::srgba(1., 0., 0., 1.0),
        );

        build_axis_ring(
            parent,
            commands,
            materials,
            gizmo_entity,
            GizmoAxis::Y,
            Color::srgba(0., 1., 0., 1.),
        );

        build_axis_ring(
            parent,
            commands,
            materials,
            gizmo_entity,
            GizmoAxis::Z,
            Color::srgba(0., 0., 1., 1.),
        );

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Rotate Gizmo spawned"
        );
    } else {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Failed to spawn rotate Gizmo. Parent Entity {:?} not found or missing Transform.",
            parent
        );
    }
}

pub fn despawn_rotate_gizmo(
    commands: &mut Commands,
    query: &mut Query<(Entity, &RotateGizmo, &Children)>,
) {
    for (entity, _, _) in query.iter() {
        commands.entity(entity).try_despawn();
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Despawned Rotate Gizmo"
        );
    }
}

fn build_visual_sphere(
    target: Entity,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let sphere = Sphere::new(ROTATE_VISUAL_RADIUS).mesh().ico(7).unwrap();
    let sphere_handle = meshes.add(sphere);
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.6, 0.6, 0.24),
        unlit: true,
        alpha_mode: AlphaMode::AlphaToCoverage,
        ..Default::default()
    });

    commands.spawn((
        Mesh3d(sphere_handle),
        MeshMaterial3d(material.clone()),
        Transform::default(),
        NotShadowCaster,
        NotShadowReceiver,
        Pickable {
            is_hoverable: true,
            should_block_lower: false,
        },
        Name::new("Gizmo Visual Sphere"),
        GizmoAxis::None,
        RotateGizmo,
        ChildOf(parent),
        GizmoOf(target),
        GizmoRoot(parent),
    ));
}

fn build_free_sphere(
    target: Entity,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    color: Color,
    meshes: &mut ResMut<Assets<Mesh>>,
) {
    let sphere = Sphere::new(ROTATE_INNER_RADIUS).mesh().ico(5).unwrap();
    let sphere_handle = meshes.add(sphere);
    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::AlphaToCoverage,
        ..Default::default()
    });

    commands
        .spawn((
            Mesh3d(sphere_handle),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            NotShadowCaster,
            NotShadowReceiver,
            Name::new("Gizmo Rotate Sphere"),
            GizmoAxis::All,
            RotateGizmo,
            GizmoMesh,
            ChildOf(parent),
            GizmoOf(target),
            GizmoRoot(parent),
        ))
        .observe(super::drag::handle_rotate_dragging);
}

fn build_axis_ring(
    target: Entity,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    axis: GizmoAxis,
    color: Color,
) {
    // Load the embedded ring mesh
    let ring_mesh = get_mesh_handle();

    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::AlphaToCoverage,
        ..Default::default()
    });

    let rotation = match axis {
        GizmoAxis::X => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        GizmoAxis::Y => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        GizmoAxis::Z => Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        _ => Quat::IDENTITY,
    };

    commands
        .spawn((
            Mesh3d(ring_mesh),
            MeshMaterial3d(material.clone()),
            Transform {
                scale: Vec3::ONE * GIZMO_SCALE,
                rotation,
                ..Default::default()
            },
            NotShadowCaster,
            NotShadowReceiver,
            Name::new("Gizmo Rotate Ring"),
            axis,
            RotateGizmo,
            GizmoMesh,
            GizmoOf(target),
            ChildOf(parent),
            GizmoRoot(parent),
        ))
        .observe(super::drag::handle_rotate_dragging);
}

pub fn update_gizmo_rotation_for_mode(
    mut gizmo_query: Query<(&mut Transform, &GizmoOf, &GizmoConfig), With<RotateGizmoParent>>,
    transform_query: Query<&Transform, Without<RotateGizmoParent>>,
    parent_query: Query<&ChildOf>,
) {
    for (mut gizmo_transform, gizmo_of, config) in gizmo_query.iter_mut() {
        if let Ok(entity_transform) = transform_query.get(gizmo_of.0) {
            match config.mode() {
                GizmoMode::Global => {
                    // Mimic GlobalTransform
                    // We want this to happen immediately, but GlobalTransform propagates later. So this is a workaround so we dont get gizmo off by one rotation
                    let mut global_rotation = entity_transform.rotation;
                    let mut current_entity = gizmo_of.0;

                    while let Ok(parent_of) = parent_query.get(current_entity) {
                        let parent_entity = parent_of.parent();
                        if let Ok(parent_transform) = transform_query.get(parent_entity) {
                            global_rotation = parent_transform.rotation * global_rotation;
                            current_entity = parent_entity;
                        } else {
                            break;
                        }
                    }

                    gizmo_transform.rotation = global_rotation.inverse();
                }
                GizmoMode::Local => {
                    gizmo_transform.rotation = Quat::IDENTITY;
                }
            }
        }
    }
}
