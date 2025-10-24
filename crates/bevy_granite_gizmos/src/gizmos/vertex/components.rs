use bevy::prelude::{Component, Entity, Vec3};

#[derive(Component)]
pub struct VertexMarker {
    pub parent_entity: Entity,
    pub vertex_index: usize,
    pub local_position: Vec3,
}

#[derive(Component)]
pub struct SelectedVertex;

#[derive(Component)]
pub struct HasVertexVisualizations;

#[derive(Component)]
pub struct VertexVisualizationParent {
    pub source_entity: Entity,
}
