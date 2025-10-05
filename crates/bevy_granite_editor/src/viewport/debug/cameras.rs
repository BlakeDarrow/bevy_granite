use super::DebugRenderer;
use crate::editor_state::EditorState;
use bevy::{
    camera::Camera,
    color::Color,
    ecs::{entity::Entity, system::Query},
    gizmos::gizmos::Gizmos,
    prelude::{Res, With, Without},
    transform::components::GlobalTransform,
};
use bevy_granite_core::MainCamera;
use bevy_granite_gizmos::Selected;

pub fn show_camera_forward_system(
    query: Query<(Entity, &GlobalTransform, &Camera), Without<MainCamera>>,
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
    for (entity, global_transform, _camera) in query.iter() {
        if config.debug_selected_only {
            match active_query.single() {
                Ok(selected_entity) if selected_entity != entity => continue,
                Err(_) => return,
                _ => {}
            }
        }
        let beam_length = 2.5;
        let forward = global_transform.forward();
        let start = global_transform.translation();
        let end = start + forward * beam_length;
        let color = Color::srgb_from_array(config.debug_color);
        gizmos.arrow(start, end, color);
    }
}
