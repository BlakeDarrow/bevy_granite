use super::{
    components::{HasVertexVisualizations, VertexMarker, VertexVisualizationParent},
    config::VertexVisualizationConfig,
};
use crate::{
    gizmos::{GizmoType, NewGizmoType},
    selection::Selected,
};
use bevy::{
    camera::visibility::RenderLayers,
    ecs::hierarchy::ChildOf,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::{Mesh3d, VertexAttributeValues},
    pbr::MeshMaterial3d,
    picking::Pickable,
    prelude::{
        Assets, Children, Commands, Cuboid, Entity, Mesh, Meshable, Name, Query, Res, ResMut,
        StandardMaterial, Transform, Vec3, Visibility, With, Without,
    },
};
use bevy_granite_core::{EditorIgnore, TreeHiddenEntity};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

/// System that spawns vertex visualizations for all selected entities
pub fn spawn_vertex_visualizations(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<VertexVisualizationConfig>,
    gizmo_type: Res<NewGizmoType>,
    selected_entities: Query<(Entity, &Mesh3d), (With<Selected>, Without<HasVertexVisualizations>)>,
) {
    if !matches!(**gizmo_type, GizmoType::Pointer) || !config.enabled {
        return;
    }

    for (entity, mesh3d) in selected_entities.iter() {
        let Some(mesh) = meshes.get(&mesh3d.0) else {
            continue;
        };

        // Extract vertex positions
        let Some(vertex_positions) = extract_vertex_positions(mesh) else {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::Entity,
                "Failed to extract vertex positions from mesh on entity {:?}",
                entity
            );
            continue;
        };

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Spawning {} vertex markers for entity {:?}",
            vertex_positions.len(),
            entity
        );

        // Create parent entity to hold all vertex markers
        let parent = commands
            .spawn((
                Transform::default(),
                Visibility::default(),
                VertexVisualizationParent {
                    source_entity: entity,
                },
                ChildOf(entity),
                TreeHiddenEntity,
                Name::new("VertexVisualizationParent"),
            ))
            .id();

        // Spawn a cube for each vertex
        for (index, position) in vertex_positions.iter().enumerate() {
            let mesh_handle = meshes.add(
                Cuboid::new(config.vertex_size, config.vertex_size, config.vertex_size).mesh(),
            );

            // Create a unique material for each vertex so they can be colored independently
            let material = materials.add(StandardMaterial {
                base_color: config.unselected_color,
                unlit: true,
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                depth_bias: -0.1,
                ..Default::default()
            });

            commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material),
                Transform::from_translation(*position),
                Visibility::default(),
                VertexMarker {
                    parent_entity: entity,
                    vertex_index: index,
                    local_position: *position,
                },
                Pickable {
                    is_hoverable: true,
                    should_block_lower: true,
                },
                EditorIgnore::PICKING,
                NotShadowCaster,
                NotShadowReceiver,
                RenderLayers::layer(14), // Layer 14 for gizmos - always renders on top
                ChildOf(parent),
                TreeHiddenEntity,
                Name::new(format!("Vertex_{}", index)),
            ));
        }

        commands
            .entity(entity)
            .queue_silenced(|mut entity: bevy::ecs::world::EntityWorldMut| {
                entity.insert(HasVertexVisualizations);
            });
    }
}

/// System that despawns vertex visualizations when conditions aren't met
pub fn despawn_vertex_visualizations(
    mut commands: Commands,
    config: Res<VertexVisualizationConfig>,
    gizmo_type: Res<NewGizmoType>,
    vertex_parents: Query<(Entity, &VertexVisualizationParent, &Children)>,
    selected_entities: Query<(), With<Selected>>,
) {
    let should_despawn = !matches!(**gizmo_type, GizmoType::Pointer)
        || !config.enabled
        || selected_entities.is_empty();

    if should_despawn {
        for (parent_entity, viz_parent, children) in vertex_parents.iter() {
            for child in children.iter() {
                if let Ok(mut entity_commands) = commands.get_entity(*child) {
                    entity_commands.despawn();
                }
            }
            // Despawn the parent
            if let Ok(mut entity_commands) = commands.get_entity(parent_entity) {
                entity_commands.despawn();
            }
            commands
                .entity(viz_parent.source_entity)
                .remove::<HasVertexVisualizations>();
        }

        if !vertex_parents.is_empty() {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Entity,
                "Despawned vertex visualizations"
            );
        }
    }
}

/// System that despawns vertex visualizations when an entity is deselected
pub fn cleanup_deselected_entity_vertices(
    mut commands: Commands,
    vertex_parents: Query<(Entity, &VertexVisualizationParent, &Children)>,
    selected_entities: Query<Entity, With<Selected>>,
) {
    for (parent_entity, viz_parent, children) in vertex_parents.iter() {
        if selected_entities.get(viz_parent.source_entity).is_err() {
            for child in children.iter() {
                if let Ok(mut entity_commands) = commands.get_entity(*child) {
                    entity_commands.despawn();
                }
            }
            if let Ok(mut entity_commands) = commands.get_entity(parent_entity) {
                entity_commands.despawn();
            }
            commands
                .entity(viz_parent.source_entity)
                .remove::<HasVertexVisualizations>();
        }
    }
}

/// Extract unique vertex positions from a mesh
fn extract_vertex_positions(mesh: &Mesh) -> Option<Vec<Vec3>> {
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;

    if let VertexAttributeValues::Float32x3(positions) = positions {
        // Use a simple deduplication by converting to a set-like structure
        let mut unique_positions = Vec::new();
        const EPSILON: f32 = 0.0001; // Small tolerance for floating point comparison

        for [x, y, z] in positions {
            let pos = Vec3::new(*x, *y, *z);

            // Check if this position already exists (within epsilon)
            let is_duplicate = unique_positions.iter().any(|existing: &Vec3| {
                (existing.x - pos.x).abs() < EPSILON
                    && (existing.y - pos.y).abs() < EPSILON
                    && (existing.z - pos.z).abs() < EPSILON
            });

            if !is_duplicate {
                unique_positions.push(pos);
            }
        }

        Some(unique_positions)
    } else {
        None
    }
}
