use bevy::ecs::{message::MessageWriter, system::ResMut};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use crate::interface::{
    cache::entity_cache::EntityUIDataCache,
    events::{UserUpdatedComponentsEvent, UserUpdatedIdentityEvent, UserUpdatedTransformEvent},
    tabs::entity_editor::{EntityGlobalTransformData, EntityIdentityData, EntityRegisteredData},
};
//
// INFO:
// sync.rs
// Keep EntityUIDataCache and EntityEditorTabData (Right Tab Widget - Entity Editor) synced
// When user edits field (Identity, Transform, Registered) in UI, trigger needed edits to entity
// via events
//

// Keep component data (Entity Editor Tab UI) up to date with cache
pub fn update_components_from_cache(
    components_data: &mut EntityRegisteredData,
    cache: &mut ResMut<EntityUIDataCache>,
    components_updated_writer: &mut MessageWriter<UserUpdatedComponentsEvent>,
) {
    // Always update UI when entity changes
    if cache.dirty.entity_dirty {
        components_data.components.clear();
        components_data.components = cache.data.registered.components.clone();
        // Also clear any pending requests when entity changes
        components_data.registered_add_request = None;
        components_data.registered_remove_request = None;
        components_data.registered_data_changed = false;
        cache.dirty.entity_dirty = false;
    } else if cache.dirty.registered_dirty {
        // Only block overwrite if user is actively editing
        let is_user_editing = components_data.registered_data_changed
            || components_data.registered_add_request.is_some()
            || components_data.registered_remove_request.is_some();
        if !is_user_editing {
            components_data.components = cache.data.registered.components.clone();
        }
        cache.dirty.registered_dirty = false;
    }

    // Your existing event logic
    let ui_changed = components_data.registered_data_changed
        || components_data.registered_add_request.is_some()
        || components_data.registered_remove_request.is_some();

    if ui_changed {
        send_component_events_from_ui_change(components_data, cache, components_updated_writer);
    }
}

// Keep transform data (Entity Editor Tab UI) up to date with cache
pub fn update_transform_from_cache(
    global_transform_data: &mut EntityGlobalTransformData,
    cache: &mut ResMut<EntityUIDataCache>,
    transform_updated_writer: &mut MessageWriter<UserUpdatedTransformEvent>,
) {
    if cache.dirty.entity_dirty {
        global_transform_data.clear();
        global_transform_data.global_transform_data = cache.data.global_transform.clone();
        cache.dirty.entity_dirty = false;
    } else if cache.dirty.global_transform_dirty {
        if !global_transform_data.transform_data_changed {
            global_transform_data.global_transform_data = cache.data.global_transform.clone();
        }
        cache.dirty.global_transform_dirty = false;
    }

    if cache.dirty.gizmo_dirty {
        global_transform_data.gizmo_axis = cache.data.gizmo_drag.locked_axis;
        cache.dirty.gizmo_dirty = false;
    }

    // Only send events if the UI data actually changed
    if global_transform_data.transform_data_changed {
        send_transform_events_from_ui_change(
            global_transform_data,
            cache,
            transform_updated_writer,
        );
    }
}

// Keep identity data (Entity Editor Tab UI) up to date with cache
pub fn update_identity_from_cache(
    identity_data: &mut EntityIdentityData,
    cache: &mut ResMut<EntityUIDataCache>,
    identity_updated_writer: &mut MessageWriter<UserUpdatedIdentityEvent>,
) {
    if cache.dirty.identity_dirty && !identity_data.name_changed {
        identity_data.name = cache.data.identity.name.to_string();
    }

    if cache.dirty.identity_dirty && !identity_data.class_data_changed {
        identity_data.class_data = cache.data.identity.class.clone();
    }

    if cache.dirty.material_dirty && !identity_data.class_data_changed {
        identity_data.class_data = cache.data.identity.class.clone();
    }

    if cache.dirty.identity_dirty {
        cache.dirty.identity_dirty = false;
    }

    send_identity_events_from_ui_change(identity_data, cache, identity_updated_writer);
}

// (Entity Editor Tab UI) Component changed via UI, update the entity
pub fn send_component_events_from_ui_change(
    registered_data: &mut EntityRegisteredData,
    cache: &mut ResMut<EntityUIDataCache>,
    components_updated_writer: &mut MessageWriter<UserUpdatedComponentsEvent>,
) {
    if let Some(entity) = cache.data.entity {
        components_updated_writer.write(UserUpdatedComponentsEvent {
            entity,
            data: registered_data.clone(),
        });
    }

    registered_data.registered_data_changed = false;
    registered_data.registered_remove_request = None;
    registered_data.registered_add_request = None;
}

// (Entity Editor Tab UI) Transformed changed via UI, update the entity
pub fn send_transform_events_from_ui_change(
    global_transform_data: &mut EntityGlobalTransformData,
    cache: &mut ResMut<EntityUIDataCache>,
    transform_updated_writer: &mut MessageWriter<UserUpdatedTransformEvent>,
) {
    if global_transform_data.transform_data_changed {
        if let Some(entity) = cache.data.entity {
            transform_updated_writer.write(UserUpdatedTransformEvent {
                entity,
                data: global_transform_data.clone(),
            });
        }

        global_transform_data.transform_data_changed = false;
    }
}

// (Entity Editor Tab UI) Identity changed via UI, update the entity
pub fn send_identity_events_from_ui_change(
    identity_data: &mut EntityIdentityData,
    cache: &mut ResMut<EntityUIDataCache>,
    identity_updated_writer: &mut MessageWriter<UserUpdatedIdentityEvent>,
) {
    if identity_data.name_changed || identity_data.class_data_changed {
        if let Some(entity) = cache.data.entity {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Entity,
                "Identity data changed"
            );

            identity_updated_writer.write(UserUpdatedIdentityEvent {
                entity,
                data: identity_data.clone(),
            });
        }
    }

    if identity_data.name_changed {
        identity_data.name_changed = false;
    }

    if identity_data.class_data_changed {
        identity_data.class_data_changed = false;
    }
}
