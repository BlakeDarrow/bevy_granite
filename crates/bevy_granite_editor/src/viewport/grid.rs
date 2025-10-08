use crate::{editor_state::EditorState, viewport::camera::LAYER_GRID};
use bevy::{
    color::Color,
    gizmos::gizmos::Gizmos,
    math::Vec3,
    prelude::{
        Commands, Component, GlobalTransform, Name, Query, Res,
        Transform, Visibility, With,
    },
};
use bevy_granite_core::{TreeHiddenEntity, UICamera};

#[derive(Component)]
pub struct ViewportGrid;

const MIN_CELL_SIZE: f32 = 0.0001;
const GRID_EPSILON: f32 = 0.0001;
const GRID_HEIGHT_OFFSET: f32 = 0.0;
const LINE_SEGMENT_LENGTH: f32 = 10.0; // Break lines into segments to avoid thickness issues

pub fn spawn_viewport_grid(
    mut commands: Commands,
) {
    commands.spawn((
        Name::new("Viewport Grid"),
        ViewportGrid,
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
        Visibility::Hidden,
        TreeHiddenEntity,
        bevy::camera::visibility::RenderLayers::layer(LAYER_GRID),
    ));
}

pub fn update_grid_system(
    mut gizmos: Gizmos,
    mut grid_query: Query<&mut Visibility, With<ViewportGrid>>,
    camera_query: Query<&bevy::transform::components::GlobalTransform, With<UICamera>>,
    editor_state: Res<EditorState>,
) {
    let Ok(mut visibility) = grid_query.single_mut() else {
        return;
    };

    if !editor_state.active || !editor_state.config.viewport.grid {
        *visibility = Visibility::Hidden;
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        *visibility = Visibility::Hidden;
        return;
    };

    let max_distance = editor_state.config.viewport.grid_distance.max(MIN_CELL_SIZE);
    let cell_size = editor_state
        .config
        .viewport
        .grid_size
        .max(MIN_CELL_SIZE);
    let color = editor_state.config.viewport.grid_color;
    let grid_color = Color::linear_rgba(color[0], color[1], color[2], color[3]);

    *visibility = Visibility::Visible;

    draw_grid_lines(&mut gizmos, camera_transform, max_distance, cell_size, grid_color);
}

fn draw_grid_lines(
    gizmos: &mut Gizmos,
    camera_transform: &bevy::transform::components::GlobalTransform,
    max_distance: f32,
    cell_size: f32,
    color: Color,
) {
    let camera_pos = camera_transform.translation();
    let center_x = camera_pos.x;
    let center_z = camera_pos.z;

    let start_x = center_x - max_distance;
    let end_x = center_x + max_distance;
    let start_z = center_z - max_distance;
    let end_z = center_z + max_distance;

    let first_x = (start_x / cell_size).floor() * cell_size;
    let first_z = (start_z / cell_size).floor() * cell_size;

    let y = GRID_HEIGHT_OFFSET;

    let mut x = first_x;
    while x <= end_x + GRID_EPSILON {
        let distance = (camera_pos - Vec3::new(x, 0.0, center_z)).length();
        if distance <= max_distance + cell_size {
            render_grid_line(
                gizmos,
                Vec3::new(x, y, start_z),
                Vec3::new(x, y, end_z),
                color,
            );
        }
        x += cell_size;
    }

    let mut z = first_z;
    while z <= end_z + GRID_EPSILON {
        let distance = (camera_pos - Vec3::new(center_x, 0.0, z)).length();
        if distance <= max_distance + cell_size {
            render_grid_line(
                gizmos,
                Vec3::new(start_x, y, z),
                Vec3::new(end_x, y, z),
                color,
            );
        }
        z += cell_size;
    }
}

// Render line using segments to avoid thickness issues
fn render_grid_line(gizmos: &mut Gizmos, start: Vec3, end: Vec3, color: Color) {
    let direction = (end - start).normalize();
    let total_length = (end - start).length();
    let mut current_distance = 0.0;

    while current_distance < total_length {
        let segment_start = start + direction * current_distance;
        let segment_end = start + direction * (current_distance + LINE_SEGMENT_LENGTH).min(total_length);
        
        gizmos.line(segment_start, segment_end, color);
        
        current_distance += LINE_SEGMENT_LENGTH;
    }
}
