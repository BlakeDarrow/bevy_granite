use bevy::prelude::{Component, Entity, Vec3};

/// Marks a vertex visualization sphere
#[derive(Component)]
pub struct VertexMarker {
    pub parent_entity: Entity,
    pub vertex_index: usize,
    pub local_position: Vec3,
}

/// Marks a vertex as selected
#[derive(Component)]
pub struct SelectedVertex;

/// Marks an entity that has vertex visualizations spawned for it
#[derive(Component)]
pub struct HasVertexVisualizations;

/// Parent entity that holds all vertex markers for one mesh
#[derive(Component)]
pub struct VertexVisualizationParent {
    pub source_entity: Entity,
}
