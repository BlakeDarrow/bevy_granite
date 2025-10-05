use crate::interface::events::MaterialHandleUpdateEvent;
use bevy::{
    ecs::{entity::Entity, message::MessageReader, system::Query},
    prelude::ResMut,
};
use bevy_granite_core::{entities::GraniteType, AvailableEditableMaterials, IdentityData};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

pub fn update_material_handle_system(
    mut material_handle_update_reader: MessageReader<MaterialHandleUpdateEvent>,
    mut _available_materials: ResMut<AvailableEditableMaterials>,
    mut identity_query: Query<(Entity, &mut IdentityData)>,
) {
    // Material handles update across all entities, so we need to rebuild the OBJ with new info
    for MaterialHandleUpdateEvent {
        skip_entity,
        version: _version,
        path,
        material,
    } in material_handle_update_reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Asset,
            "Heard world material handle update"
        );
        let mut changed: u64 = 0;
        for (entity, mut identity) in identity_query.iter_mut() {
            if entity == *skip_entity {
                continue;
            }
            if let Some(material_data) = identity.class.get_mut_material_data() {
                if material_data.current.path == *path && material.disk_changes {
                    changed += 1;
                    // Someone else changed our EditableMaterial, lets copy it
                    *material_data.current = material.clone();
                }
            }
        }
        if changed > 0 {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Asset,
                "Updated material '{}' on '{}' non-active entities",
                material.path,
                changed
            );
        }
    }
}
