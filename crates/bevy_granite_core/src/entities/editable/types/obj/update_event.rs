use super::UserUpdatedOBJEvent;
use crate::{entities::editable::RequestEntityUpdateFromClass, NeedsTangents, OBJ};
use bevy::{
    asset::AssetServer,
    ecs::{entity::Entity, event::EventReader, system::{Commands, Res}},
    prelude::Query,
    render::mesh::Mesh3d,
};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

impl OBJ {
    pub fn push_to_entity(
        &self,
        entity: Entity,
        request_update: &mut RequestEntityUpdateFromClass,
    ) {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Requesting OBJ entity update"
        );

        request_update.obj.write(UserUpdatedOBJEvent {
            entity,
            data: self.clone(),
            reload_mesh: self.reload_requested,
        });
    }
}

pub fn update_obj_system(
    mut reader: EventReader<UserUpdatedOBJEvent>,
    mesh_query: Query<&Mesh3d>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for UserUpdatedOBJEvent {
        entity: requested_entity,
        data: new_obj_data,
        reload_mesh,
    } in reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Heard obj update event: {}",
            requested_entity
        );

        if *reload_mesh {
            if let Ok(_mesh3d) = mesh_query.get(*requested_entity) {
                // Force reload the asset by path
                asset_server.reload(new_obj_data.mesh_path.as_ref());
                commands.entity(*requested_entity).insert(NeedsTangents);
                
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Asset,
                    "Successfully triggered reload for OBJ mesh entity {}: {}",
                    requested_entity,
                    new_obj_data.mesh_path
                );
            } else {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::Asset,
                    "Failed to find Mesh3d component for entity {} during OBJ reload",
                    requested_entity
                );
            }
        }
        // Handle other OBJ updates here if needed in the future
    }
}
