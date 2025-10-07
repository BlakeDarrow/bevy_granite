use crate::{editor_state::EditorState, viewport::camera::LAYER_GRID};
use bevy::{
    asset::RenderAssetUsages,
    math::Vec3,
    mesh::{Indices, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        AlphaMode, Assets, Color, Commands, Component, GlobalTransform, Mesh, Name, Query, Res,
        ResMut, Transform, Visibility, With,
    },
    render::render_resource::PrimitiveTopology,
};
use bevy_granite_core::{EditorIgnore, UICamera};

#[derive(Component)]
pub struct ViewportGrid;

const MIN_CELL_SIZE: f32 = 0.0001;
const MIN_LINE_THICKNESS: f32 = 0.0005;
const GRID_EPSILON: f32 = 0.0001;
const GRID_HEIGHT_OFFSET: f32 = 0.0;
const GRID_DEPTH_BIAS: f32 = 1000.0; // Ensure grid renders on top of most things without z-fighting

pub fn spawn_viewport_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    ));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.6, 0.6, 0.5),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        fog_enabled: false,
        cull_mode: None,
        depth_bias: GRID_DEPTH_BIAS,
        ..Default::default()
    });

    commands.spawn((
        Name::new("Viewport Grid"),
        ViewportGrid,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
        Visibility::Hidden,
        EditorIgnore::PICKING,
        bevy::camera::visibility::RenderLayers::layer(LAYER_GRID),
    ));
}

pub fn update_grid_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid_query: Query<
        (
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &mut Visibility,
        ),
        With<ViewportGrid>,
    >,
    camera_query: Query<&bevy::transform::components::GlobalTransform, With<UICamera>>,
    editor_state: Res<EditorState>,
) {
    let Ok((mesh_handle, material_handle, mut visibility)) = grid_query.single_mut() else {
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
    let line_thickness = (cell_size * 0.05).max(MIN_LINE_THICKNESS);
    let color = editor_state.config.viewport.grid_color;

    if let Some(material) = materials.get_mut(&material_handle.0) {
        material.base_color = Color::srgba(color[0], color[1], color[2], color[3]);
        material.alpha_mode = AlphaMode::Blend;
        material.unlit = true;
        material.fog_enabled = false;
        material.cull_mode = None;
        material.depth_bias = GRID_DEPTH_BIAS;
    }

    let (positions, normals, uvs, indices) =
        build_grid_geometry(camera_transform, max_distance, cell_size, line_thickness);

    if positions.is_empty() {
        *visibility = Visibility::Hidden;
        return;
    }

    *visibility = Visibility::Visible;

    if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
        let mut new_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        new_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        new_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        new_mesh.insert_indices(Indices::U32(indices));
        *mesh = new_mesh;
    }
}

fn build_grid_geometry(
    camera_transform: &bevy::transform::components::GlobalTransform,
    max_distance: f32,
    cell_size: f32,
    line_thickness: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let camera_pos = camera_transform.translation();
    let center_x = camera_pos.x;
    let center_z = camera_pos.z;

    let start_x = center_x - max_distance;
    let end_x = center_x + max_distance;
    let start_z = center_z - max_distance;
    let end_z = center_z + max_distance;

    let first_x = (start_x / cell_size).floor() * cell_size;
    let first_z = (start_z / cell_size).floor() * cell_size;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let mut add_quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| {
        let base_index = positions.len() as u32;
        positions.push(a.to_array());
        positions.push(b.to_array());
        positions.push(c.to_array());
        positions.push(d.to_array());

        let normal = Vec3::Y;
        normals.extend([normal.to_array(); 4]);
        uvs.extend([[0.0, 0.0]; 4]);

        indices.extend_from_slice(&[
            base_index,
            base_index + 2,
            base_index + 1,
            base_index + 2,
            base_index + 3,
            base_index + 1,
        ]);
    };

    let half_thickness = line_thickness * 0.5;
    let mut x = first_x;
    while x <= end_x + GRID_EPSILON {
        let distance = (camera_pos - Vec3::new(x, 0.0, center_z)).length();
        if distance <= max_distance + cell_size {
            let y = GRID_HEIGHT_OFFSET;
            let a = Vec3::new(x - half_thickness, y, start_z);
            let b = Vec3::new(x + half_thickness, y, start_z);
            let c = Vec3::new(x - half_thickness, y, end_z);
            let d = Vec3::new(x + half_thickness, y, end_z);
            add_quad(a, b, c, d);
        }
        x += cell_size;
    }

    let mut z = first_z;
    while z <= end_z + GRID_EPSILON {
        let distance = (camera_pos - Vec3::new(center_x, 0.0, z)).length();
        if distance <= max_distance + cell_size {
            let y = GRID_HEIGHT_OFFSET;
            let a = Vec3::new(start_x, y, z - half_thickness);
            let b = Vec3::new(start_x, y, z + half_thickness);
            let c = Vec3::new(end_x, y, z - half_thickness);
            let d = Vec3::new(end_x, y, z + half_thickness);
            add_quad(a, b, c, d);
        }
        z += cell_size;
    }

    (positions, normals, uvs, indices)
}
