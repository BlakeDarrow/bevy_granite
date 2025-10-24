use super::{
    components::{SelectedVertex, VertexMarker},
    config::VertexSelectionState,
};
use bevy::prelude::{GlobalTransform, Query, ResMut, Vec3, With};

pub fn calculate_vertex_midpoint(
    mut selection_state: ResMut<VertexSelectionState>,
    selected_vertices: Query<&GlobalTransform, (With<VertexMarker>, With<SelectedVertex>)>,
) {
    if selection_state.selected_vertices.len() < 2 {
        selection_state.midpoint_world = None;
        return;
    }

    let mut sum = Vec3::ZERO;
    let mut count = 0;

    for global_transform in selected_vertices.iter() {
        sum += global_transform.translation();
        count += 1;
    }

    if count > 0 {
        let midpoint = sum / count as f32;
        selection_state.midpoint_world = Some(midpoint);
    } else {
        selection_state.midpoint_world = None;
    }
}
