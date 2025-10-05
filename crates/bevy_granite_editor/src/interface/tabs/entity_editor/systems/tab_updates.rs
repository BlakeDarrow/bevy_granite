use crate::interface::{
    cache::{
        entity_cache::EntityUIDataCache, sync::update_identity_from_cache,
        update_components_from_cache, update_transform_from_cache,
    },
    events::{
        MaterialDeleteEvent, UserUpdatedComponentsEvent, UserUpdatedIdentityEvent,
        UserUpdatedTransformEvent,
    },
    panels::right_panel::{SideDockState, SideTab},
    tabs::entity_editor::EntityIdentityData,
};
use bevy::ecs::{
    change_detection::DetectChanges,
    entity::Entity,
    message::MessageWriter,
    query::With,
    system::{Query, Res, ResMut},
};
use bevy_granite_core::{
    entities::GraniteType, AvailableEditableMaterials, ComponentEditor, RegisteredTypeNames,
};
use bevy_granite_gizmos::ActiveSelection;

// Every frame we check the tab for staleness
pub fn update_entity_editor_tab_system(
    mut right_dock: ResMut<SideDockState>,
    mut cache: ResMut<EntityUIDataCache>,
    type_names: Res<RegisteredTypeNames>,
    mut available_materials: ResMut<AvailableEditableMaterials>,
    mut active_entity: Query<Entity, With<ActiveSelection>>,
    mut identity_updated_writer: MessageWriter<UserUpdatedIdentityEvent>,
    mut transform_updated_writer: MessageWriter<UserUpdatedTransformEvent>,
    mut component_updated_writer: MessageWriter<UserUpdatedComponentsEvent>,
    mut material_delete_writer: MessageWriter<MaterialDeleteEvent>,
    global_component_editor: ResMut<ComponentEditor>,
) {
    for (_, tab) in right_dock.dock_state.iter_all_tabs_mut() {
        if let SideTab::EntityEditor { ref mut data } = tab {
            // Handle material deletion requests FIRST to avoid borrowing conflicts
            if data.material_delete_requested {
                data.material_delete_requested = false;

                let current_path =
                    if let Some(mat_data) = data.identity_data.class_data.get_material_data() {
                        mat_data.current.path.clone()
                    } else {
                        String::new()
                    };

                if !current_path.is_empty() {
                    // Delete from disk and memory using a temporary clone
                    let temp_material =
                        if let Some(mat_data) = data.identity_data.class_data.get_material_data() {
                            mat_data.current.clone()
                        } else {
                            continue;
                        };

                    temp_material.delete_from_disk_and_memory(&mut available_materials);

                    // Reset current material to "None" (index 0)
                    if let Some(materials) = &available_materials.materials {
                        if !materials.is_empty() {
                            if let Some(mat_data) =
                                data.identity_data.class_data.get_mut_material_data()
                            {
                                *mat_data.current = materials[0].clone();
                                *mat_data.path = materials[0].path.clone();
                            }

                            // Send deletion event to notify other systems
                            material_delete_writer
                                .write(MaterialDeleteEvent { path: current_path });
                        }
                    }
                }
            }

            let has_selected = cache.data.entity;
            let identity_data = &mut data.identity_data;
            let components_data = &mut data.registered_data;
            let active = &mut data.active_entity;

            let global_transform_data = &mut data.global_transform_data;

            // Only clear data when entity selection changes from some entity to none
            if has_selected != data.last_selected_entity {
                if has_selected.is_none() {
                    // Entity was deselected - clear all data
                    *identity_data = EntityIdentityData::default();
                    global_transform_data.clear();
                    components_data.clear();
                    *active = None;
                }
                data.last_selected_entity = has_selected;
            }

            if has_selected.is_some() {
                if let Ok(single_entity) = active_entity.single_mut() {
                    *active = Some(single_entity);
                }
            }

            if data.component_editor.is_none() {
                data.component_editor = Some(global_component_editor.clone());
            }

            if available_materials.is_changed()
                || (data.available_materials.materials.is_none()
                    && available_materials.materials.is_some())
            {
                data.available_materials = available_materials.as_ref().clone();
            }

            update_identity_from_cache(identity_data, &mut cache, &mut identity_updated_writer);
            update_transform_from_cache(
                global_transform_data,
                &mut cache,
                &mut transform_updated_writer,
            );
            update_components_from_cache(
                components_data,
                &mut cache,
                &mut component_updated_writer,
            );

            // FIX: Do this on init, not here. this is dirty?
            if !data.init {
                data.registered_type_names = type_names.names.clone();
                data.init = true;
            }
        }
    }
}
