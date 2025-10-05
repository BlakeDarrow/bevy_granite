use super::IconEntity;
use crate::{editor_state::EditorState, viewport::config::VisualizationConfig};
use bevy::{
    camera::visibility::RenderLayers,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::{Indices, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        Assets, Commands, Entity, Handle, Image, Mesh, Name, Query, Res, ResMut, Transform, Without,
    },
    render::alpha::AlphaMode,
};
use bevy_granite_core::{GraniteType, IconProxy, IdentityData, TreeHiddenEntity};

pub fn spawn_icon_entities_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    entity_query: Query<(Entity, &IdentityData), Without<IconEntity>>,
    existing_icons_query: Query<&IconEntity>,
    editor_state: Res<EditorState>,
) {
    if !editor_state.active {
        return;
    }
    let config = &editor_state.config.viewport.visualizers;
    if !config.icons_enabled {
        return;
    }

    // Get a set of entities that already have icons
    let entities_with_icons: std::collections::HashSet<Entity> = existing_icons_query
        .iter()
        .map(|icon| icon.target_entity)
        .collect();

    // Spawn icons for all entities based on their class type
    for (entity, identity_data) in entity_query.iter() {
        if !entities_with_icons.contains(&entity) {
            let class = &identity_data.class;

            let handle = class.get_icon_handle();
            let name = class.type_name() + "_icon";

            if let Some(handle) = handle {
                spawn_icon_for_entity(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    entity,
                    handle,
                    name,
                    config,
                );
            }
        }
    }
}

fn spawn_icon_for_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    target_entity: Entity,
    texture_handle: Handle<Image>,
    icon_name: String,
    config: &VisualizationConfig,
) {
    // Create a quad mesh for the icon
    let quad_mesh = create_icon_quad_mesh();
    let mesh_handle = meshes.add(quad_mesh);

    // Create material with the embedded icon texture
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        base_color: bevy::color::Color::srgb_from_array(config.icon_color),
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.5), // Use alpha cutout for icons
        cull_mode: None,
        ..Default::default()
    });

    let icon_entity = commands
        .spawn((
            IconEntity { target_entity },
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Name::new(icon_name),
            Transform::default(), // Offset relative to parent
            TreeHiddenEntity,
            IconProxy { target_entity },
            RenderLayers::from_layers(&[14]), // 14 is our UI/Gizmo layer.
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .id();

    // Make the icon a child of the target entity
    commands.entity(target_entity).add_child(icon_entity);
}

fn create_icon_quad_mesh() -> Mesh {
    let half_size = 0.5; // Always unit quad

    // Quad vertices (facing +Z initially) - counter-clockwise winding
    let vertices = vec![
        [-half_size, -half_size, 0.0], // bottom-left
        [half_size, -half_size, 0.0],  // bottom-right
        [half_size, half_size, 0.0],   // top-right
        [-half_size, half_size, 0.0],  // top-left
    ];

    // UV coordinates
    let uvs = vec![
        [0.0, 1.0], // bottom-left
        [1.0, 1.0], // bottom-right
        [1.0, 0.0], // top-right
        [0.0, 0.0], // top-left
    ];

    // Normals (all facing +Z)
    let normals = vec![
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];

    // Triangle indices (counter-clockwise winding for front face)
    let indices = vec![
        0, 1, 2, // first triangle
        2, 3, 0, // second triangle
    ];

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        Default::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
