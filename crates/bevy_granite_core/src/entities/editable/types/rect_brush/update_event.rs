use crate::entities::editable::RequestEntityUpdateFromClass;

use super::{RectBrush, UserUpdatedRectBrushEvent};
use bevy::{
    asset::Assets,
    camera::primitives::{Aabb, MeshAabb},
    ecs::{
        message::MessageReader,
        system::{Query, ResMut},
    },
    mesh::Mesh3d,
};
use bevy::{mesh::Mesh, prelude::Entity};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

impl RectBrush {
    pub fn push_to_entity(
        &self,
        rect_e: Entity,
        request_update: &mut RequestEntityUpdateFromClass,
    ) {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Requesting Rectangle Brush update"
        );
        request_update
            .rectangle_brush
            .write(UserUpdatedRectBrushEvent {
                entity: rect_e,
                data: self.clone(),
            });
    }
}

pub fn update_rectangle_brush_system(
    mut reader: MessageReader<UserUpdatedRectBrushEvent>,
    mut query: Query<(Entity, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut aabbs: Query<&mut Aabb>,
) {
    for UserUpdatedRectBrushEvent {
        entity: requested_entity,
        data: new,
    } in reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Heard rectangle brush update event: {}",
            requested_entity
        );
        if let Ok((_entity, mesh_handle)) = query.get_mut(*requested_entity) {
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                let new_vertices = RectBrush::get_vertices(new.size.x, new.size.y, new.size.z);
                let new_uvs = RectBrush::get_uvs(new.uv_scale);
                let indices = RectBrush::get_indices();
                let normals = RectBrush::calculate_normals(&new_vertices, &indices);

                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_vertices);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, new_uvs);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                // Compute the new AABB from the updated mesh
                if let Some(new_mesh_aabb) = mesh.compute_aabb() {
                    if let Ok(mut entity_aabb) = aabbs.get_mut(*requested_entity) {
                        *entity_aabb = new_mesh_aabb;

                        log!(
                            LogType::Editor,
                            LogLevel::Info,
                            LogCategory::Entity,
                            "Updated entity AABB for {}: min={:?}, max={:?}",
                            requested_entity,
                            new_mesh_aabb.min(),
                            new_mesh_aabb.max()
                        );
                    } else {
                        log!(
                            LogType::Editor,
                            LogLevel::Warning,
                            LogCategory::Entity,
                            "Entity {} has no AABB component to update",
                            requested_entity
                        );
                    }
                } else {
                    log!(
                        LogType::Editor,
                        LogLevel::Error,
                        LogCategory::Entity,
                        "Failed to compute AABB for updated mesh"
                    );
                }

                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Updated rectangle brush mesh for entity {}",
                    requested_entity
                );
            }
        } else {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Entity,
                "Could not find rectangle brush entity {}",
                requested_entity
            );
        }
    }
}
