use super::{
    components::{SelectedVertex, VertexMarker},
    config::{VertexSelectionState, VertexVisualizationConfig},
};
use bevy::{
    ecs::observer::On,
    pbr::MeshMaterial3d,
    picking::events::{Click, Pointer},
    prelude::{Commands, Entity, KeyCode, Query, Res, ResMut, StandardMaterial, With, Without},
};
use bevy_granite_core::UserInput;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

pub fn handle_vertex_click(
    mut event: On<Pointer<Click>>,
    mut commands: Commands,
    user_input: Res<UserInput>,
    vertex_query: Query<(Entity, &VertexMarker)>,
    selected_vertices: Query<Entity, With<SelectedVertex>>,
    mut selection_state: ResMut<VertexSelectionState>,
) {
    let clicked_entity = event.entity;

    let Ok((vertex_entity, vertex_marker)) = vertex_query.get(clicked_entity) else {
        return;
    };

    event.propagate(false);

    let is_additive = user_input.current_button_inputs.iter().any(|input| {
        matches!(
            input,
            bevy_granite_core::InputTypes::Button(KeyCode::ShiftLeft | KeyCode::ShiftRight)
        )
    });

    if !is_additive {
        for entity in selected_vertices.iter() {
            commands.entity(entity).remove::<SelectedVertex>();
        }
        selection_state.selected_vertices.clear();
    }

    commands.entity(vertex_entity).insert(SelectedVertex);
    selection_state.selected_vertices.push(vertex_entity);

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Entity,
        "Selected vertex {} on entity {:?}",
        vertex_marker.vertex_index,
        vertex_marker.parent_entity
    );
}

pub fn update_vertex_colors(
    config: Res<VertexVisualizationConfig>,
    selected_vertices: Query<&MeshMaterial3d<StandardMaterial>, With<SelectedVertex>>,
    unselected_vertices: Query<
        &MeshMaterial3d<StandardMaterial>,
        (With<VertexMarker>, Without<SelectedVertex>),
    >,
    mut materials: ResMut<bevy::prelude::Assets<StandardMaterial>>,
) {
    for material_handle in selected_vertices.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = config.selected_color;
        }
    }

    for material_handle in unselected_vertices.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = config.unselected_color;
        }
    }
}

pub fn deselect_all_vertices(
    mut commands: Commands,
    selected_vertices: Query<Entity, With<SelectedVertex>>,
    mut selection_state: ResMut<VertexSelectionState>,
    user_input: Res<UserInput>,
) {
    let should_deselect = user_input.current_button_inputs.iter().any(|input| {
        matches!(
            input,
            bevy_granite_core::InputTypes::Button(KeyCode::Escape)
        )
    });

    if should_deselect {
        for entity in selected_vertices.iter() {
            commands.entity(entity).remove::<SelectedVertex>();
        }
        selection_state.selected_vertices.clear();
        selection_state.midpoint_world = None;

        if !selected_vertices.is_empty() {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Entity,
                "Deselected all vertices"
            );
        }
    }
}
