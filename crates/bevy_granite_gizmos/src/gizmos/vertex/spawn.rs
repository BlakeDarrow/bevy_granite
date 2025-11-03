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
        Assets, Camera, Children, Commands, Entity, Mesh, Meshable, Name, Query, Res, ResMut,
        Sphere, StandardMaterial, Transform, Vec3, Visibility, With, Without,
    },
};
use bevy_granite_core::{EditorIgnore, TreeHiddenEntity, UICamera};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

/// System that spawns vertex visualizations for all selected entities
pub fn spawn_vertex_visualizations(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut config: ResMut<VertexVisualizationConfig>,
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

        let shared_mesh = meshes.add(
            Sphere::new(config.vertex_size).mesh().ico(2).unwrap(),
        );
        
        let unselected_material = if let Some(mat) = &config.unselected_material {
            mat.clone()
        } else {
            let mat = materials.add(StandardMaterial {
                base_color: config.unselected_color,
                unlit: true,
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                depth_bias: -0.1,
                ..Default::default()
            });
            config.unselected_material = Some(mat.clone());
            mat
        };
        
        let selected_material = if let Some(mat) = &config.selected_material {
            mat.clone()
        } else {
            let mat = materials.add(StandardMaterial {
                base_color: config.selected_color,
                unlit: true,
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                depth_bias: -0.1,
                ..Default::default()
            });
            config.selected_material = Some(mat.clone());
            mat
        };

        // Spawn vertices using shared resources
        for (index, position) in vertex_positions.iter().enumerate() {
            commands.spawn((
                Mesh3d(shared_mesh.clone()),
                MeshMaterial3d(unselected_material.clone()),
                Transform::from_translation(*position),
                Visibility::Visible,
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
        let mut unique_positions = Vec::new();
        const EPSILON: f32 = 0.0001; 

        for [x, y, z] in positions {
            let pos = Vec3::new(*x, *y, *z);

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

/// System that culls vertex visualizations based on distance from camera
/// and scales them so they appear smaller as you zoom in
pub fn cull_vertices_by_distance(
    config: Res<VertexVisualizationConfig>,
    camera_query: Query<&Transform, (With<Camera>, With<UICamera>)>,
    mut vertex_query: Query<(&mut Transform, &mut Visibility, &VertexMarker), Without<Camera>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let camera_pos = camera_transform.translation;
    let max_distance_sq = config.max_distance * config.max_distance;

    for (mut vertex_transform, mut visibility, _marker) in vertex_query.iter_mut() {
        let vertex_world_pos = vertex_transform.translation;
        let distance = camera_pos.distance(vertex_world_pos);
        let distance_sq = distance * distance;
        
        if distance_sq > max_distance_sq {
            *visibility = Visibility::Hidden;
            continue;
        }
        
        *visibility = Visibility::Visible;
        
        // Scale vertices with distance to maintain consistent screen size
        // Further away = bigger scale to compensate for perspective
        let scale_factor = (distance * 0.5).max(0.5).min(15.0);
        vertex_transform.scale = Vec3::splat(scale_factor);
    }
}
