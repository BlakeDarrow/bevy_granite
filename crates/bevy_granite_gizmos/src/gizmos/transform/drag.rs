use super::TransformGizmo;
use crate::{
    gizmos::{GizmoOf, GizmoSnap},
    input::GizmoAxis,
    GizmoCamera,
};
use bevy::{
    asset::Assets,
    ecs::{component::Component, observer::Trigger, system::Commands},
    gizmos::{retained::Gizmo, GizmoAsset},
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
    picking::events::{Drag, Pointer, Pressed},
    prelude::{Entity, GlobalTransform, Query, Res, ResMut, Transform, Vec3, With},
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

pub fn drag_transform_gizmo(
    event: Trigger<Pointer<Drag>>,
    targets: Query<&GizmoOf>,
    camera_query: Query<
        (Entity, &GlobalTransform, &bevy::render::camera::Camera),
        With<GizmoCamera>,
    >,
    mut objects: Query<&mut Transform>,
    gizmo_snap: Res<GizmoSnap>,
    gizmo_data: Query<(&GizmoAxis, &TransformGizmo)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok((axis, typ)) = gizmo_data.get(event.target) else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "Gizmo Axis data not found for Gizmo entity {:?}",
            event.target
        );
        return;
    };

    let Ok((c_entity, camera_transform, camera)) = camera_query.single() else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo camera not found",
        };
        return;
    };

    let Ok(GizmoOf(target)) = targets.get(event.target) else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo target not found for entity {:?}",
            event.target
        };
        return;
    };
    let Ok(click_ray) = camera.viewport_to_world(camera_transform, event.pointer_location.position)
    else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Failed to convert viewport to world coordinates for pointer location: {:?}",
            event.pointer_location.position
        };
        return;
    };

    let Ok(mut target_transform) = objects.get_mut(*target) else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo target transform not found for entity {:?}",
            target
        };
        return;
    };

    let start = target_transform.translation;
    match (axis, typ) {
        (GizmoAxis::None, _) => {}
        (GizmoAxis::X, TransformGizmo::Axis) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(0., target_transform.translation.y, 0.),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::Y),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            target_transform.translation.x = snap_gizmo(hit.x, gizmo_snap.transform_value);
        }
        (GizmoAxis::Y, TransformGizmo::Axis) => {
            let mut normal = camera_transform.forward().as_vec3();
            normal.y = 0.0;
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(
                    target_transform.translation.x,
                    0.,
                    target_transform.translation.z,
                ),
                bevy::math::primitives::InfinitePlane3d::new(normal.normalize()),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            target_transform.translation.y = snap_gizmo(hit.y, gizmo_snap.transform_value);
        }
        (GizmoAxis::Z, TransformGizmo::Axis) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(0., target_transform.translation.y, 0.),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::Y),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            target_transform.translation.z = snap_gizmo(hit.z, gizmo_snap.transform_value);
        }
        (GizmoAxis::X, TransformGizmo::Plane) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(target_transform.translation.x, 0., 0.),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::X),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            target_transform.translation.y = snap_gizmo(hit.y, gizmo_snap.transform_value);
            target_transform.translation.z = snap_gizmo(hit.z, gizmo_snap.transform_value);
        }
        (GizmoAxis::Y, TransformGizmo::Plane) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(0., target_transform.translation.y, 0.),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::Y),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            target_transform.translation.x = snap_gizmo(hit.x, gizmo_snap.transform_value);
            target_transform.translation.z = snap_gizmo(hit.z, gizmo_snap.transform_value);
        }
        (GizmoAxis::Z, TransformGizmo::Plane) => {
            let Some(click_distance) = click_ray.intersect_plane(
                Vec3::new(0., 0., target_transform.translation.z),
                bevy::math::primitives::InfinitePlane3d::new(Vec3::Z),
            ) else {
                return;
            };
            let hit = camera_transform.translation() - (click_ray.direction * -click_distance);
            target_transform.translation.x = snap_gizmo(hit.x, gizmo_snap.transform_value);
            target_transform.translation.y = snap_gizmo(hit.y, gizmo_snap.transform_value);
        }
        (GizmoAxis::All, _) => {
            let Some(_click_distance) = click_ray.intersect_plane(
                target_transform.translation,
                bevy::math::primitives::InfinitePlane3d::new(camera_transform.forward()),
            ) else {
                return;
            };

            // let hit = camera_transform.translation() + (click_ray.direction * click_distance);
            // target_transform.translation = hit;
        }
    }
    if input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight) {
        let delta = target_transform.translation - start;
        if let Ok(mut camera_transform) = objects.get_mut(c_entity) {
            camera_transform.translation += delta;
        }
    }
}

fn snap_gizmo(value: f32, inc: f32) -> f32 {
    if inc == 0.0 {
        value
    } else {
        (value / inc).round() * inc
    }
}

pub fn draw_axis_lines(
    event: Trigger<Pointer<Pressed>>,
    gizmo_data: Query<(&GizmoAxis, &GizmoOf, &TransformGizmo), With<TransformGizmo>>,
    mut bevy_gizmo: ResMut<Assets<GizmoAsset>>,
    mut commands: Commands,
    origin: Query<&GlobalTransform>,
) {
    let Ok((axis, root, transform)) = gizmo_data.get(event.target) else {
        return;
    };
    if let GizmoAxis::All = axis {
        return;
    }
    let Ok(origin) = origin.get(root.get()) else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo origin transform not found for entity {:?}",
            root.0
        };
        return;
    };
    let mut asset = GizmoAsset::new();
    match transform {
        TransformGizmo::Axis => {
            asset.line(
                origin.translation() + axis.to_vec3() * 1000.,
                origin.translation() + axis.to_vec3() * -1000.,
                axis.color(),
            );
        }
        TransformGizmo::Plane => {
            let (a, b) = axis.plane();
            asset.line(
                origin.translation() + a.to_vec3() * 1000.,
                origin.translation() + a.to_vec3() * -1000.,
                a.color(),
            );
            asset.line(
                origin.translation() + b.to_vec3() * 1000.,
                origin.translation() + b.to_vec3() * -1000.,
                b.color(),
            );
        }
    }

    commands.spawn((
        *axis,
        GizmoOf(root.0),
        Gizmo {
            handle: bevy_gizmo.add(asset),
            ..Default::default()
        },
        AxisLine,
    ));
}

pub fn cleanup_axis_line(
    mut commands: Commands,
    query: Query<Entity, With<AxisLine>>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.just_released(MouseButton::Left) {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct AxisLine;
