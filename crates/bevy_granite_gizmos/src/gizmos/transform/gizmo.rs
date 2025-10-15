use bevy::{
    ecs::hierarchy::{ChildOf, Children},
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::Mesh3d,
    pbr::MeshMaterial3d,
    prelude::{
        AlphaMode, Assets, Color, Commands, Component, Cone, Cylinder, Entity, GlobalTransform,
        Mesh, Meshable, Name, Quat, Query, ResMut, Resource, Sphere, StandardMaterial, Transform,
        Vec3, Visibility, With, Without,
    },
};
use bevy_granite_core::TreeHiddenEntity;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use crate::{
    gizmos::{GizmoConfig, GizmoMesh, GizmoMode, GizmoOf, GizmoRoot},
    input::GizmoAxis,
};

#[derive(Component, Debug)]
pub enum TransformGizmo {
    Axis,
    Plane,
}

#[derive(Resource, Default, Component)]
pub struct TransformGizmoParent;

#[derive(Resource, Default)]
pub struct PreviousTransformGizmo {
    pub entity: Option<Entity>,
}

const GIZMO_SCALE: f32 = 1.35;

const TRANSFORM_INNER_RADIUS: f32 = 0.09 * GIZMO_SCALE; // middle sphere of gizmo
const TRANSFORM_LINE_LENGTH: f32 = 0.6 * GIZMO_SCALE; // length of line
const TRANSFORM_LINE_WIDTH: f32 = 0.04 * GIZMO_SCALE; // width of line
const TRANSFORM_HANDLE_LENGTH: f32 = 0.22 * GIZMO_SCALE; // cone handle length
const TRANSFORM_HANDLE_WIDTH: f32 = 0.09 * GIZMO_SCALE; // cone handle width

pub fn spawn_transform_gizmo(
    parent: Entity,
    query: &mut Query<&GlobalTransform, Without<TransformGizmoParent>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: GizmoConfig,
) {
    let offset = Vec3::new(0., 0., 0.);

    if let Ok(parent_global_transform) = query.get(parent) {
        let gizmo_translation = offset;

        let gizmo_entity = commands
            .spawn((
                Transform {
                    translation: gizmo_translation,
                    rotation: parent_global_transform
                        .to_scale_rotation_translation()
                        .1
                        .inverse(),
                    ..Default::default()
                },
                Visibility::default(),
                GizmoOf(parent),
                ChildOf(parent),
                config,
            ))
            .insert(Name::new("TransformGizmo"))
            .insert(TransformGizmoParent)
            .insert(TreeHiddenEntity)
            .id();

        build_gizmo_sphere(
            parent,
            commands,
            meshes,
            materials,
            gizmo_entity,
            GizmoAxis::All,
            Color::srgba(0.8, 0.8, 0.8, 1.),
        );

        build_axis_cylinder(
            parent,
            commands,
            meshes,
            materials,
            gizmo_entity,
            GizmoAxis::X,
            Color::srgba(1., 0., 0., 1.),
        );
        build_axis_cylinder(
            parent,
            commands,
            meshes,
            materials,
            gizmo_entity,
            GizmoAxis::Y,
            Color::srgba(0., 1., 0., 1.),
        );
        build_axis_cylinder(
            parent,
            commands,
            meshes,
            materials,
            gizmo_entity,
            GizmoAxis::Z,
            Color::srgba(0., 0., 1., 1.),
        );
        build_axis_plane(
            parent,
            commands,
            meshes,
            materials,
            gizmo_entity,
            GizmoAxis::X,
            Color::srgba(1., 0., 0., 1.),
        );
        build_axis_plane(
            parent,
            commands,
            meshes,
            materials,
            gizmo_entity,
            GizmoAxis::Y,
            Color::srgba(0., 1., 0., 1.),
        );
        build_axis_plane(
            parent,
            commands,
            meshes,
            materials,
            gizmo_entity,
            GizmoAxis::Z,
            Color::srgba(0., 0., 1., 1.),
        );
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Transform Gizmo spawned"
        );
    } else {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Failed to spawn Transform Gizmo. Parent Entity {:?} not found or missing Transform.",
            parent
        );
    }
}

fn build_gizmo_sphere(
    root: Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    axis: GizmoAxis,
    color: Color,
) {
    let sphere = Sphere::new(TRANSFORM_INNER_RADIUS).mesh().ico(5).unwrap();
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
            MeshMaterial3d(material),
            NotShadowCaster,
            NotShadowReceiver,
            Name::from("Gizmo Transform Sphere".to_string()),
            axis,
            TransformGizmo::Axis,
            GizmoMesh,
            GizmoOf(root),
            ChildOf(parent),
        ))
        .observe(super::drag::calculate_drag_offset)
        .observe(super::drag::drag_end_cleanup)
        .observe(super::drag::drag_transform_gizmo)
        .observe(super::drag::dragstart_transform_gizmo);
}

fn build_axis_cylinder(
    root: Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    axis: GizmoAxis,
    color: Color,
) {
    let arrow_mesh = meshes.add(Mesh::from(Cylinder {
        radius: TRANSFORM_LINE_WIDTH,
        half_height: TRANSFORM_LINE_LENGTH * 0.5,
    }));

    let cone_mesh = meshes.add(Mesh::from(Cone {
        radius: TRANSFORM_HANDLE_WIDTH,
        height: TRANSFORM_HANDLE_LENGTH,
    }));

    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::AlphaToCoverage,
        ..Default::default()
    });

    commands
        .spawn((
            Mesh3d(arrow_mesh),
            MeshMaterial3d(material.clone()),
            Transform {
                translation: axis.to_vec3() * TRANSFORM_LINE_LENGTH * 0.5,
                rotation: Quat::from_rotation_arc(Vec3::Y, axis.to_vec3()),
                ..Default::default()
            },
            NotShadowCaster,
            NotShadowReceiver,
            Name::new("Gizmo Transform Cone"),
            GizmoRoot(parent),
        ))
        .insert(GizmoOf(root))
        .insert(axis)
        .insert(TransformGizmo::Axis)
        .insert(GizmoMesh)
        .insert(ChildOf(parent))
        .observe(super::drag::calculate_drag_offset)
        .observe(super::drag::drag_end_cleanup)
        .observe(super::drag::drag_transform_gizmo)
        .observe(super::drag::dragstart_transform_gizmo)
        .with_children(|p| {
            p.spawn((
                Mesh3d(cone_mesh),
                MeshMaterial3d(material.clone()),
                Transform {
                    translation: Vec3::Y * (TRANSFORM_LINE_LENGTH * 0.5),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
                Name::new("Gizmo Transform Arrow"),
                GizmoOf(root),
                GizmoRoot(parent),
                axis,
                TransformGizmo::Axis,
                GizmoMesh,
            ));
        });
}

fn build_axis_plane(
    root: Entity,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
    axis: GizmoAxis,
    color: Color,
) {
    let plane_mesh = meshes.add(Mesh::from(bevy::prelude::Cuboid::new(
        TRANSFORM_LINE_LENGTH * 0.33,
        TRANSFORM_LINE_WIDTH,
        TRANSFORM_LINE_LENGTH * 0.33,
    )));

    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });

    commands
        .spawn((
            Mesh3d(plane_mesh),
            MeshMaterial3d(material.clone()),
            Transform {
                translation: plane_translation(axis),
                rotation: plane_rotation(axis),
                ..Default::default()
            },
            NotShadowCaster,
            NotShadowReceiver,
            Name::new("Gizmo Transform Plane"),
            GizmoOf(root),
            GizmoRoot(parent),
            axis,
            TransformGizmo::Plane,
            GizmoMesh,
            ChildOf(parent),
        ))
        .observe(super::drag::calculate_drag_offset)
        .observe(super::drag::drag_end_cleanup)
        .observe(super::drag::drag_transform_gizmo)
        .observe(super::drag::dragstart_transform_gizmo);
}

pub fn despawn_transform_gizmo(
    commands: &mut Commands,
    query: &mut Query<(Entity, &TransformGizmoParent, &Children)>,
) {
    for (entity, _, _) in query.iter() {
        commands.entity(entity).try_despawn();
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Despawned Transform Gizmo"
        );
    }
}

fn plane_translation(axis: GizmoAxis) -> Vec3 {
    let offset = TRANSFORM_LINE_LENGTH * 0.33;
    match axis {
        GizmoAxis::X => Vec3::new(0., offset, offset),
        GizmoAxis::Y => Vec3::new(offset, 0., offset),
        GizmoAxis::Z => Vec3::new(offset, offset, 0.),
        _ => Vec3::ZERO,
    }
}

fn plane_rotation(axis: GizmoAxis) -> Quat {
    match axis {
        GizmoAxis::X => Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        GizmoAxis::Y => Quat::IDENTITY,
        GizmoAxis::Z => Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        _ => Quat::IDENTITY,
    }
}

pub fn update_gizmo_rotation_for_mode(
    mut gizmo_query: Query<(&mut Transform, &GizmoOf, &GizmoConfig), With<TransformGizmoParent>>,
    parent_query: Query<&GlobalTransform>,
) {
    for (mut gizmo_transform, gizmo_of, config) in gizmo_query.iter_mut() {
        if let Ok(parent_global_transform) = parent_query.get(gizmo_of.0) {
            let parent_rotation = parent_global_transform.to_scale_rotation_translation().1;
            
            match config.mode() {
                GizmoMode::Global => {
                    gizmo_transform.rotation = parent_rotation.inverse();
                }
                GizmoMode::Local => {
                    gizmo_transform.rotation = Quat::IDENTITY;
                }
            }
        }
    }
}
