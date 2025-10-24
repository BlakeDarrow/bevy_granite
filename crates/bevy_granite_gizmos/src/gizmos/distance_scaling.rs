use bevy::{
    ecs::{
        query::With,
        system::{Query, Res, ResMut},
    },
    math::Vec3,
    transform::components::{GlobalTransform, Transform},
};

use super::{GizmoChildren, GizmoType};
use crate::{
    gizmos::{NewGizmoType, GizmoConfig},
    NewGizmoConfig,
};
use bevy_granite_core::UICamera;

const DISTANCE_SCALING_ENABLED: bool = true;

pub fn scale_gizmo_by_camera_distance_system(
    camera_q: Query<&GlobalTransform, With<UICamera>>,
    mut gizmo_q: Query<
        (&GlobalTransform, &mut Transform, Option<&mut GizmoConfig>),
        With<GizmoChildren>,
    >,
    selected_gizmo: Res<NewGizmoType>,
    mut default_config: ResMut<NewGizmoConfig>,
) {
    if !DISTANCE_SCALING_ENABLED {
        return;
    }

    let Ok(cam_transform) = camera_q.single() else {
        return;
    };

    if matches!(**selected_gizmo, GizmoType::Pointer) {
        return;
    }

    if gizmo_q.is_empty() {
        return;
    }

    for (gizmo_global_transform, mut gizmo_transform, config) in gizmo_q.iter_mut() {
        // Calculate what local scale is needed to achieve global scale of 1.0
        let current_global_scale = gizmo_global_transform.to_scale_rotation_translation().0;
        let parent_scale_factor = current_global_scale / gizmo_transform.scale;
        let baseline_local_scale = Vec3::splat(1.0) / parent_scale_factor;

        let distance = cam_transform
            .translation()
            .distance(gizmo_global_transform.translation());

        let base_scale = 0.5; 
        let distance_scale = 0.08; 
        let min_scale = 0.01; 
        let scale_factor = (distance * distance_scale).clamp(min_scale, 9.0); 
        let final_scale = (base_scale * scale_factor).clamp(min_scale, f32::INFINITY);

        // Apply the final scale while accounting for parent transforms
        gizmo_transform.scale = baseline_local_scale * final_scale;
        default_config.distance_scale = final_scale;
        if let Some(mut config) = config {
            match config.as_mut() {
                GizmoConfig::Transform {
                    ref mut distance_scale,
                    ..
                } => {
                    *distance_scale = final_scale;
                }
                GizmoConfig::Rotate {
                    ref mut distance_scale,
                    ref mut speed_scale,
                    ..
                } => {
                    *distance_scale = final_scale;
                    if final_scale > base_scale {
                        *speed_scale = (final_scale * 1.01).clamp(1.0, 1.5);
                    } else {
                        *speed_scale = 1.0;
                    }
                }
                GizmoConfig::None | GizmoConfig::Pointer => {}
            };
        } else {
            // Transform Gizmo should have higher upper limit on speed
            match **selected_gizmo {
                GizmoType::Pointer | GizmoType::None => {
                    default_config.speed_scale = 1.0;
                }
                GizmoType::Transform => {
                    if final_scale > base_scale {
                        default_config.speed_scale = final_scale * 3.3; // the further away we are how much faster should entities be moved
                    } else {
                        default_config.speed_scale = 1.0
                    }
                }
                GizmoType::Rotate => {
                    if final_scale > base_scale {
                        default_config.speed_scale = (final_scale * 1.01).clamp(1.0, 1.5);
                    } else {
                        default_config.speed_scale = 1.0
                    }
                }
            };
        };
    }
}
