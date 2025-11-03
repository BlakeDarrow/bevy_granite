use bevy::prelude::{Color, Entity, Handle, Resource, StandardMaterial, Vec3};

#[derive(Resource)]
pub struct VertexVisualizationConfig {
    pub enabled: bool,
    pub vertex_size: f32,
    pub max_distance: f32,
    pub unselected_color: Color,
    pub selected_color: Color,
    pub highlight_color: Color,
    pub unselected_material: Option<Handle<StandardMaterial>>,
    pub selected_material: Option<Handle<StandardMaterial>>,
}

impl Default for VertexVisualizationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            vertex_size: 0.004,
            max_distance: 10.0,
            unselected_color: Color::srgba(1., 1., 1., 1.0),
            selected_color: Color::srgba(1.0, 0.8, 0.0, 1.0),
            highlight_color: Color::srgba(0.9, 0.9, 0.9, 1.0),
            unselected_material: None,
            selected_material: None,
        }
    }
}

#[derive(Resource, Default)]
pub struct VertexSelectionState {
    pub selected_vertices: Vec<Entity>,
    pub midpoint_world: Option<Vec3>,
}
