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
    mut selected_vertices: Query<&mut MeshMaterial3d<StandardMaterial>, (With<SelectedVertex>, With<VertexMarker>)>,
    mut unselected_vertices: Query<&mut MeshMaterial3d<StandardMaterial>, (With<VertexMarker>, Without<SelectedVertex>)>,
) {
    // Swap material handles to selected material for selected vertices
    if let Some(selected_mat) = &config.selected_material {
        for mut material in selected_vertices.iter_mut() {
            material.0 = selected_mat.clone();
        }
    }
    
    // Swap material handles to unselected material for unselected vertices
    if let Some(unselected_mat) = &config.unselected_material {
        for mut material in unselected_vertices.iter_mut() {
            material.0 = unselected_mat.clone();
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
