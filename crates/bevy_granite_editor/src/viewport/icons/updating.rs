use super::IconEntity;
use crate::editor_state::EditorState;
use bevy::{
    camera::visibility::Visibility,
    color::Color,
    math::Vec3,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{Assets, Entity, Query, Res, ResMut, Transform, With, Without},
    transform::components::GlobalTransform,
};
use bevy_granite_core::UICamera;
use bevy_granite_gizmos::{gizmos::NewGizmoType, ActiveSelection, DragState, GizmoType, Selected};

pub fn update_icon_entities_system(
    mut icon_query: Query<
        (
            &IconEntity,
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
        ),
        With<IconEntity>,
    >,
    drag_state: Res<DragState>,
    selected_gizmo: Res<NewGizmoType>,
    active_query: Query<Entity, With<ActiveSelection>>,
    mut selected_query: Query<Entity, (With<Selected>, Without<ActiveSelection>)>,
    target_query: Query<&GlobalTransform, Without<IconEntity>>,
    camera_query: Query<&GlobalTransform, (With<UICamera>, Without<IconEntity>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    editor_state: Res<EditorState>,
) {
    let rotating = drag_state.dragging && matches!(**selected_gizmo, GizmoType::Rotate);
    if !editor_state.active {
        // Hide all icons when editor is disabled
        for (_, _, mut visibility, _) in icon_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    let config = &editor_state.config.viewport.visualizers;

    let camera_transform = match camera_query.single() {
        Ok(cam) => cam,
        Err(_) => return,
    };

    for (icon_entity, mut icon_transform, mut visibility, material_handle) in icon_query.iter_mut()
    {
        // Hide icons if disabled
        if !config.icons_enabled {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }

        // Hide active icon if not showing active icons
        if !config.icon_show_active {
            if let Ok(target) = active_query.single() {
                if icon_entity.target_entity == target {
                    *visibility = Visibility::Hidden;
                }
            }
        }

        // Hide active when rotating
        if config.icon_show_active {
            if let Ok(target) = active_query.single() {
                if icon_entity.target_entity == target {
                    if rotating {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
        }

        if !config.icon_show_selected {
            for e in selected_query.iter_mut() {
                if icon_entity.target_entity == e {
                    *visibility = Visibility::Hidden;
                }
            }
        }

        // Update icon material color
        if let Some(material) = materials.get_mut(material_handle) {
            let mut color = Color::srgb_from_array(config.icon_color);
            if let Ok(target) = active_query.single() {
                if icon_entity.target_entity == target {
                    color = Color::srgb_from_array(config.selection_active_color);
                }
            }

            for e in selected_query.iter_mut() {
                if icon_entity.target_entity == e {
                    color = Color::srgb_from_array(config.selection_color);
                }
            }

            material.base_color = color;
        }

        if let Ok(target_global_transform) = target_query.get(icon_entity.target_entity) {
            // No need to update position - the parent-child relationship handles that automatically!

            // Calculate distance-based scaling if enabled
            let distance =
                (camera_transform.translation() - target_global_transform.translation()).length();
            let scale = if config.icon_distance_scaling {
                let scale_factor = (distance / 10.0).clamp(0.5, 3.0);
                scale_factor * config.icon_size
            } else {
                config.icon_size
            };

            let camera_pos = camera_transform.translation();
            let icon_pos = target_global_transform.translation();
            let direction_to_camera = (camera_pos - icon_pos).normalize();

            let up = Vec3::Y;
            let right = up.cross(direction_to_camera).normalize();
            let corrected_up = direction_to_camera.cross(right).normalize();

            // Calculate world-space billboard rotation
            let world_billboard_rotation = bevy::math::Quat::from_mat3(
                &bevy::math::Mat3::from_cols(right, corrected_up, direction_to_camera),
            );

            // Convert to local space by removing parent's rotation
            let parent_rotation = target_global_transform.to_scale_rotation_translation().1;
            icon_transform.rotation = parent_rotation.inverse() * world_billboard_rotation;

            // Convert to local space by removing parent's rotation
            let parent_rotation = target_global_transform.to_scale_rotation_translation().1;
            icon_transform.rotation = parent_rotation.inverse() * world_billboard_rotation;

            // Apply world down offset only when entity is selected - gets visually messy
            let is_active = active_query
                .single()
                .map_or(false, |target| icon_entity.target_entity == target);
            //let is_selected = selected_query.iter().any(|e| icon_entity.target_entity == e);

            let world_down_offset = if is_active {
                Vec3::new(0.0, -0.35, 0.0)
            } else {
                Vec3::ZERO
            };

            let local_down_offset = parent_rotation.inverse() * world_down_offset;
            icon_transform.translation = local_down_offset;

            icon_transform.scale = Vec3::splat(scale);

            // Hide icon if too far away
            if config.icon_max_distance > 0.0 && distance > config.icon_max_distance {
                icon_transform.scale = Vec3::ZERO;
            }
        }
    }
}
