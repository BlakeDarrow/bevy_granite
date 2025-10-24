use bevy::prelude::{Color, Entity, Resource, Vec3};

#[derive(Resource)]
pub struct VertexVisualizationConfig {
    pub enabled: bool,
    pub vertex_size: f32,
    pub unselected_color: Color,
    pub selected_color: Color,
    pub highlight_color: Color,
}

impl Default for VertexVisualizationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            vertex_size: 0.008,
            unselected_color: Color::srgba(0.7, 0.7, 0.7, 1.0),
            selected_color: Color::srgba(1.0, 0.8, 0.0, 1.0),
            highlight_color: Color::srgba(0.9, 0.9, 0.9, 1.0),
        }
    }
}

#[derive(Resource, Default)]
pub struct VertexSelectionState {
    pub selected_vertices: Vec<Entity>,
    pub midpoint_world: Option<Vec3>,
}
