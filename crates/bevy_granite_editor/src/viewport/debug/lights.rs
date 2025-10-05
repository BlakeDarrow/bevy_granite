use super::DebugRenderer;
use crate::editor_state::EditorState;
use bevy::{
    color::Color,
    ecs::{entity::Entity, system::Query},
    gizmos::gizmos::Gizmos,
    light::{DirectionalLight, PointLight},
    prelude::{Res, With},
    transform::components::GlobalTransform,
};
use bevy_granite_gizmos::Selected;

pub fn show_directional_light_forward_system(
    query: Query<(Entity, &GlobalTransform, &DirectionalLight)>,
    active_query: Query<Entity, With<Selected>>,
    mut gizmos: Gizmos<DebugRenderer>,
    editor_state: Res<EditorState>,
) {
    if !editor_state.active {
        return;
    }
    let config = editor_state.config.viewport.visualizers;
    if !config.debug_enabled {
        return;
    }
    for (entity, global_transform, _directional_light) in query.iter() {
        if config.debug_selected_only {
            match active_query.single() {
                Ok(selected_entity) if selected_entity != entity => continue,
                Err(_) => return,
                _ => {}
            }
        }
        let beam_length = 3.5;
        let forward = global_transform.forward();
        let start = global_transform.translation();
        let end = start + forward * beam_length;
        let color = Color::srgb_from_array(config.debug_color);
        gizmos.arrow(start, end, color);

        let sun_radius = 0.3;
        gizmos.circle(start, sun_radius, color);
    }
}

pub fn show_point_light_range_system(
    query: Query<(Entity, &GlobalTransform, &PointLight)>,
    active_query: Query<Entity, With<Selected>>,
    mut gizmos: Gizmos<DebugRenderer>,
    editor_state: Res<EditorState>,
) {
    if !editor_state.active {
        return;
    }
    let config = editor_state.config.viewport.visualizers;
    if !config.debug_enabled {
        return;
    }
    for (entity, global_transform, point_light) in query.iter() {
        if config.debug_selected_only {
            match active_query.single() {
                Ok(selected_entity) if selected_entity != entity => continue,
                Err(_) => return,
                _ => {}
            }
        }
        let range = point_light.range;
        let pos = global_transform.to_isometry();
        let color = Color::srgb_from_array(config.debug_color);
        gizmos.sphere(pos, range, color);
    }
}
