use super::{
    components::{SelectedVertex, VertexMarker},
    config::{VertexSelectionState, VertexVisualizationConfig},
};
use bevy::{
    ecs::observer::On,
    pbr::MeshMaterial3d,
    picking::events::{Click, Pointer},
    prelude::{
        Commands, Entity, KeyCode, Query, Res, ResMut, StandardMaterial, With, Without,
    },
};
use bevy_granite_core::UserInput;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

/// System that handles clicking on vertices
pub fn handle_vertex_click(
    mut event: On<Pointer<Click>>,
    mut commands: Commands,
    user_input: Res<UserInput>,
    vertex_query: Query<(Entity, &VertexMarker)>,
    selected_vertices: Query<Entity, With<SelectedVertex>>,
    mut selection_state: ResMut<VertexSelectionState>,
) {
    let clicked_entity = event.entity;

    // Check if the clicked entity is a vertex marker
    let Ok((vertex_entity, vertex_marker)) = vertex_query.get(clicked_entity) else {
        return;
    };

    // Stop event propagation so other systems don't handle this click
    event.propagate(false);

    // Check if shift is held for additive selection
    let is_additive = user_input
        .current_button_inputs
        .iter()
        .any(|input| matches!(input, bevy_granite_core::InputTypes::Button(KeyCode::ShiftLeft | KeyCode::ShiftRight)));

    if !is_additive {
        // Clear all previously selected vertices
        for entity in selected_vertices.iter() {
            commands.entity(entity).remove::<SelectedVertex>();
        }
        selection_state.selected_vertices.clear();
    }

    // Select this vertex
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

/// System that updates vertex colors based on selection state
pub fn update_vertex_colors(
    config: Res<VertexVisualizationConfig>,
    selected_vertices: Query<&MeshMaterial3d<StandardMaterial>, With<SelectedVertex>>,
    unselected_vertices: Query<
        &MeshMaterial3d<StandardMaterial>,
        (With<VertexMarker>, Without<SelectedVertex>),
    >,
    mut materials: ResMut<bevy::prelude::Assets<StandardMaterial>>,
) {
    // Update selected vertices to selected color
    for material_handle in selected_vertices.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = config.selected_color;
        }
    }

    // Update unselected vertices to default color
    for material_handle in unselected_vertices.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color = config.unselected_color;
        }
    }
}

/// System to deselect all vertices (e.g., when clicking in empty space)
pub fn deselect_all_vertices(
    mut commands: Commands,
    selected_vertices: Query<Entity, With<SelectedVertex>>,
    mut selection_state: ResMut<VertexSelectionState>,
    user_input: Res<UserInput>,
) {
    // Check for Escape key or other deselect conditions
    let should_deselect = user_input
        .current_button_inputs
        .iter()
        .any(|input| matches!(input, bevy_granite_core::InputTypes::Button(KeyCode::Escape)));

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
