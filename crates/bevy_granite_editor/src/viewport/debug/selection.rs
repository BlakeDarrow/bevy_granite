use crate::{editor_state::EditorState, get_entity_bounds_or_fallback};

use super::SelectionRenderer;
use bevy::{
    color::Color,
    ecs::{entity::Entity, system::Query},
    gizmos::gizmos::Gizmos,
    math::Vec3,
    prelude::{Assets, Mesh, Res, With},
    mesh::Mesh3d,
    transform::components::GlobalTransform,
};
use bevy_granite_core::IdentityData;
use bevy_granite_gizmos::{ActiveSelection, Selected};

fn should_skip_bounds_rendering(editor_state: &EditorState) -> bool {
    !editor_state.active || !editor_state.config.viewport.visualizers.selection_enabled
}

pub fn show_active_selection_bounds_system(
    query: Query<(Entity, &GlobalTransform, &IdentityData)>,
    mesh_query: Query<&Mesh3d>,
    active_query: Query<Entity, With<ActiveSelection>>,
    mut gizmos: Gizmos<SelectionRenderer>,
    meshes: Res<Assets<Mesh>>,
    editor_state: Res<EditorState>,
) {
    if should_skip_bounds_rendering(&editor_state) {
        return;
    }
    let config = editor_state.config.viewport.visualizers;
    for (entity, transform, identity_data) in query.iter() {
        match active_query.single() {
            Ok(selected_entity) if selected_entity != entity => continue,
            Err(_) => return,
            _ => {}
        }

        let offset = config.selection_bounds_offset;
        let length = config.selection_corner_length;
        let color = Color::srgb_from_array(config.selection_active_color);

        if let Some((min, max)) =
            get_entity_bounds_or_fallback(entity, identity_data, &meshes, &mesh_query)
        {
            draw_bounds_box(&mut gizmos, min, max, transform, color, offset, length);
        }
    }
}

pub fn show_selected_entities_bounds_system(
    query: Query<(Entity, &GlobalTransform, &IdentityData), With<Selected>>,
    mesh_query: Query<&Mesh3d>,
    mut gizmos: Gizmos<SelectionRenderer>,
    meshes: Res<Assets<Mesh>>,
    editor_state: Res<EditorState>,
) {
    if should_skip_bounds_rendering(&editor_state) {
        return;
    }
    let config = editor_state.config.viewport.visualizers;

    let selected_entities: Vec<_> = query.iter().collect();
    if selected_entities.is_empty() {
        return;
    }

    let mut global_min = Vec3::splat(f32::INFINITY);
    let mut global_max = Vec3::splat(f32::NEG_INFINITY);
    let mut valid_count = 0;

    // When multiple entities are selected, we show bounds for all of them
    let multiple_selected = selected_entities.len() > 1;

    for (entity, global_transform, identity_data) in selected_entities {
        valid_count += 1;

        let (local_min, local_max) = if let Some((min, max)) =
            get_entity_bounds_or_fallback(entity, identity_data, &meshes, &mesh_query)
        {
            (min, max)
        } else {
            continue;
        };

        let local_corners = [
            Vec3::new(local_min.x, local_min.y, local_min.z),
            Vec3::new(local_max.x, local_min.y, local_min.z),
            Vec3::new(local_max.x, local_min.y, local_max.z),
            Vec3::new(local_min.x, local_min.y, local_max.z),
            Vec3::new(local_min.x, local_max.y, local_min.z),
            Vec3::new(local_max.x, local_max.y, local_min.z),
            Vec3::new(local_max.x, local_max.y, local_max.z),
            Vec3::new(local_min.x, local_max.y, local_max.z),
        ];

        for corner in local_corners {
            let world_corner = global_transform.transform_point(corner);
            global_min = global_min.min(world_corner);
            global_max = global_max.max(world_corner);
        }
    }

    // Show bounds if we have any valid entities when multiple are selected, or 2+ for single type filtering
    let should_show_bounds = if multiple_selected {
        valid_count > 0 && global_min.x != f32::INFINITY
    } else {
        valid_count >= 2 && global_min.x != f32::INFINITY
    };

    if !should_show_bounds {
        return;
    }

    let offset = config.selection_bounds_offset + 0.1;
    let length = 0.5;
    let color = Color::srgb_from_array(config.selection_color);

    let identity_transform = GlobalTransform::IDENTITY;
    draw_bounds_box(
        &mut gizmos,
        global_min,
        global_max,
        &identity_transform,
        color,
        offset,
        length,
    );
}

fn draw_bounds_box(
    gizmos: &mut Gizmos<SelectionRenderer>,
    min: Vec3,
    max: Vec3,
    transform: &GlobalTransform,
    color: Color,
    offset: f32,
    length_factor: f32, // % of edge
) {
    let expanded_min = min - Vec3::splat(offset);
    let expanded_max = max + Vec3::splat(offset);

    let corners = [
        Vec3::new(expanded_min.x, expanded_min.y, expanded_min.z),
        Vec3::new(expanded_max.x, expanded_min.y, expanded_min.z),
        Vec3::new(expanded_max.x, expanded_min.y, expanded_max.z),
        Vec3::new(expanded_min.x, expanded_min.y, expanded_max.z),
        Vec3::new(expanded_min.x, expanded_max.y, expanded_min.z),
        Vec3::new(expanded_max.x, expanded_max.y, expanded_min.z),
        Vec3::new(expanded_max.x, expanded_max.y, expanded_max.z),
        Vec3::new(expanded_min.x, expanded_max.y, expanded_max.z),
    ];

    // Transform all corners to world space
    let world_corners: Vec<Vec3> = corners
        .iter()
        .map(|&corner| transform.transform_point(corner))
        .collect();

    for (i, &corner) in world_corners.iter().enumerate() {
        let adjacent_indices = match i {
            0 => [1, 3, 4],
            1 => [0, 2, 5],
            2 => [1, 3, 6],
            3 => [0, 2, 7],
            4 => [0, 5, 7],
            5 => [1, 4, 6],
            6 => [2, 5, 7],
            7 => [3, 4, 6],
            _ => continue,
        };

        for &adj_idx in &adjacent_indices {
            let diff = world_corners[adj_idx] - corner;
            let edge_length = diff.length();

            if edge_length > f32::EPSILON {
                let direction = diff / edge_length;
                let line_len = edge_length * length_factor;
                gizmos.line(corner, corner + direction * line_len, color);
            }
        }
    }
}
