use crate::interface::{
    cache::EntityCacheQueryItem,
    events::{
        MaterialDeleteEvent, MaterialHandleUpdateEvent, UserUpdatedComponentsEvent,
        UserUpdatedIdentityEvent, UserUpdatedTransformEvent,
    },
    tabs::entity_editor::EntityRegisteredData,
};
use bevy::{
    asset::AssetServer,
    color::Color,
    ecs::{
        system::Commands,
        world::{Mut, World},
    },
    pbr::MeshMaterial3d,
    prelude::{ChildOf, Entity, Handle, Name, StandardMaterial, Transform, With},
    transform::components::GlobalTransform,
};
use bevy::{
    asset::Assets,
    ecs::{
        message::{MessageReader, MessageWriter},
        query::Without,
        system::{Query, Res, ResMut},
    },
};
use bevy_granite_core::{
    entities::{editable::RequestEntityUpdateFromClass, GraniteType, Unknown},
    AvailableEditableMaterials, ComponentEditor, EditableMaterial, EditableMaterialError,
    EditableMaterialField, IdentityData, StandardMaterialDef,
};
use bevy_granite_gizmos::GizmoChildren;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

//
// INFO:
// entity_editor_system.rs
// Every frame we check if (Identity, Transform, Registered) changes are present from user via UI
// edits or via the EntityUIDataCache. If the UI data is stale we update it from the cache, if the cache is stale we
// update it from the UI
//
// INFO:
// Systems to actually update the entity with changes are found here

// Request to change the entity transform, update the entity transform, and inverse the gizmo rotation
// so it doesn't move with the new rotation
pub fn update_entity_with_new_transform_system(
    mut transform_updated_reader: MessageReader<UserUpdatedTransformEvent>,
    mut e_query: Query<
        (Entity, &mut Transform, &GlobalTransform, Option<&ChildOf>),
        (Without<GizmoChildren>, With<IdentityData>),
    >,
    mut g_query: Query<(Entity, &mut Transform), (With<GizmoChildren>, Without<IdentityData>)>,
    parent_query: Query<&GlobalTransform, (With<IdentityData>, Without<GizmoChildren>)>,
) {
    for UserUpdatedTransformEvent { entity, data } in transform_updated_reader.read() {
        if let Ok((_, mut transform, _current_global, parent)) = e_query.get_mut(*entity) {
            let target_global = Transform {
                translation: data.global_transform_data.position,
                rotation: data.global_transform_data.rotation,
                scale: data.global_transform_data.scale,
            };

            if let Some(parent_entity) = parent {
                // Entity has a parent - need to compute local transform from desired global transform
                if let Ok(parent_global) = parent_query.get(parent_entity.parent()) {
                    // Use GlobalTransform's affine method to get the inverse
                    let parent_global_inverse = parent_global.affine().inverse();
                    let target_global_affine = GlobalTransform::from(target_global).affine();
                    let local_affine = parent_global_inverse * target_global_affine;

                    // Extract the local transform components
                    let (local_scale, local_rotation, local_translation) =
                        local_affine.to_scale_rotation_translation();

                    transform.translation = local_translation;
                    transform.rotation = local_rotation;
                    transform.scale = local_scale;
                } else {
                    // Parent not found, fall back to direct assignment
                    transform.translation = target_global.translation;
                    transform.rotation = target_global.rotation;
                    transform.scale = target_global.scale;
                }
            } else {
                // No parent - can directly assign to local transform
                transform.translation = target_global.translation;
                transform.rotation = target_global.rotation;
                transform.scale = target_global.scale;
            }
        }

        // Counteract parent rotation by setting gizmo's local transform to inverse of parent rotation
        for (_gizmo_entity, mut gizmo_transform) in g_query.iter_mut() {
            gizmo_transform.translation = bevy::math::Vec3::ZERO;
            gizmo_transform.rotation = data.global_transform_data.rotation.inverse();
            gizmo_transform.scale = bevy::math::Vec3::ONE;
        }
    }
}

pub fn update_entity_with_new_identity_system(
    mut commands: Commands,
    mut identity_updated_reader: MessageReader<UserUpdatedIdentityEvent>,
    mut material_handle_update_writer: MessageWriter<MaterialHandleUpdateEvent>,
    mut request_writer: RequestEntityUpdateFromClass,
    mut query: Query<EntityCacheQueryItem>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut available_obj_materials: ResMut<AvailableEditableMaterials>,
    asset_server: Res<AssetServer>,
) {
    for UserUpdatedIdentityEvent {
        entity: updated_entity,
        data: update_data,
    } in identity_updated_reader.read()
    {
        for (
            entity,
            name,
            _transform,
            _global_transform,
            identity_data_ref,
            mut material_handle,
            _active,
        ) in query.iter_mut()
        {
            if entity != *updated_entity {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Entity is stale, skipping updates"
                );
                continue;
            }

            let mut identity_data = if let Some(data) = identity_data_ref.as_ref() {
                data.as_ref().clone()
            } else {
                IdentityData {
                    class: bevy_granite_core::GraniteTypes::Unknown(Unknown::default()),
                    uuid: uuid::Uuid::new_v4(),
                    name: name
                        .map(|n| n.to_string())
                        .unwrap_or(format!("Entity {:?}", entity)),
                }
            };

            // Only update materials when there are actual changes, not any change
            let needs_mat_update =
                if let Some(source_data) = update_data.class_data.get_material_data() {
                    source_data.current.disk_changes
                        || source_data.current.new_material
                        || (identity_data
                            .class
                            .get_material_data()
                            .map_or(true, |target_data| {
                                target_data.current.path != source_data.current.path
                            }))
                } else {
                    false
                };
            let needs_entity_name_update = update_data.name_changed;

            // push class updates back
            identity_data.class = update_data.class_data.clone();

            if needs_entity_name_update {
                identity_data.name = update_data.name.clone();
                commands
                    .entity(entity)
                    .insert(Name::new(update_data.name.clone()));

                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Entity name stale"
                );
            }

            // Push identity data to entity
            let class = &identity_data.class;
            if class.is_known() {
                class.push_to_entity(entity, &mut request_writer);
            } else {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::UI,
                    "Could not send 'push_to_entity'. Class type unknown: {:?}",
                    class
                );
            }

            // Handle material updates for entities that support materials
            if let Some(source_data) = update_data.class_data.get_material_data() {
                // Get mutable target material data from identity
                if let Some(target_data) = identity_data.class.get_mut_material_data() {
                    // special cases for setting a material to Empty

                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::Asset,
                        "Source: {:#?}\nTarget: {:#?}",
                        source_data,
                        target_data,
                    );

                    if target_data.current.is_empty() {
                        // adding a 0 handle so it looks pink as a visual placeholder. For truly a "None" material, remove these 2 calls
                        let none_handle = Handle::<StandardMaterial>::default();
                        target_data.current.set_handle(Some(none_handle.clone()));
                        commands.entity(entity).insert(MeshMaterial3d(none_handle));
                        log!(
                            LogType::Editor,
                            LogLevel::Info,
                            LogCategory::Entity,
                            "Forcing StandardMaterial handle set to 0"
                        );
                        return;
                    }

                    handle_material_update(
                        *updated_entity,
                        source_data.current,
                        source_data.path,
                        target_data.current,
                        target_data.last,
                        needs_mat_update,
                        &mut material_handle,
                        &mut materials,
                        &mut available_obj_materials,
                        &asset_server,
                        &mut material_handle_update_writer,
                    );
                }
            }

            if let Some(mut data) = identity_data_ref {
                *data = identity_data;
            }
        }
    }
}

// Rework this, duplicated code. most can also live in EditableMaterial itself.
fn handle_material_update(
    requester: Entity,
    material: &EditableMaterial,
    new_material_path: &str,
    target_material: &mut EditableMaterial,
    last_material: &mut EditableMaterial,
    needs_mat_update: bool,
    material_handle: &mut Option<Mut<'_, MeshMaterial3d<StandardMaterial>>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    available_obj_materials: &mut ResMut<AvailableEditableMaterials>,
    asset_server: &Res<AssetServer>,
    material_handle_update_writer: &mut MessageWriter<MaterialHandleUpdateEvent>,
) {
    if !material.disk_changes && !needs_mat_update {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "No material changes detected, skipping material update"
        );
        return;
    }

    if material.disk_changes {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Material has disk changes, updating..."
        );
    }

    if material.new_material {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "New material detected, updating..."
        );
    }

    let needs_new_material = material.new_material;

    // Create new material logic
    // this should move into new impl of EditableMaterial
    if needs_new_material {
        log(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Asset,
            "We need a new material".to_string(),
        );

        let default_material = StandardMaterial {
            unlit: false,
            base_color: Color::WHITE,
            ..Default::default()
        };

        let friendly_name = std::path::Path::new(&new_material_path)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase()
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
            .collect::<String>();

        let material_def = StandardMaterialDef {
            base_color: Some((1.0, 1.0, 1.0, 1.0)),
            friendly_name: friendly_name.clone(),
            ..Default::default()
        };

        let new_material = EditableMaterial {
            path: new_material_path.to_string(),
            friendly_name: friendly_name.clone(),
            def: Some(material_def.clone()),
            error: EditableMaterialError::None,
            fields: vec![EditableMaterialField::BaseColor].into(),
            disk_changes: true,
            new_material: true,
            handle: Some(materials.add(default_material.clone())),
            version: 0,
        };

        *target_material = new_material.clone();

        target_material.update_material_handle(
            &material_def,
            materials,
            available_obj_materials,
            asset_server,
        );

        // Handle path exists error
        if target_material.error == EditableMaterialError::PathExists {
            // Revert to last material
            *target_material = last_material.clone();

            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::Entity,
                "Did not apply new material, path already exists."
            );

            // Reset material flags
            target_material.new_material = false;
            target_material.reset_errors();

            return;
        }

        // Update material handle if both handles exist
        if let Some(ref handle) = target_material.handle {
            if let Some(ref mut mat_handle) = material_handle {
                mat_handle.0 = handle.clone();
            }
        }

        if let Some(ref mut materials_list) = available_obj_materials.materials {
            materials_list.push(target_material.clone());
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Asset,
                "Material added to available materials: {:?}",
                target_material.path
            );
        }

        return;
    }

    // Update material logic
    if !needs_new_material {
        // Update material definition if provided
        if let Some(ref material_def) = material.def {
            target_material.update_material_handle(
                material_def,
                materials,
                available_obj_materials,
                asset_server,
            );
        } else {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::Entity,
                "Material definition is missing"
            );
        }

        // Update entity's material handle and send event
        if let Some(updated_handle) = &target_material.handle {
            if let Some(ref mut handle) = material_handle {
                handle.0 = updated_handle.clone();
            }
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Asset,
                "Sending material handle update to world"
            );

            material_handle_update_writer.write(MaterialHandleUpdateEvent {
                skip_entity: requester,
                path: target_material.path.clone(),
                version: target_material.version,
                material: target_material.clone(),
            });
        }
    }
}

pub fn update_entity_with_new_components_system(
    mut components_updated_reader: MessageReader<UserUpdatedComponentsEvent>,
    mut commands: Commands,
) {
    for UserUpdatedComponentsEvent { entity, data } in components_updated_reader.read() {
        let entity = *entity;
        let data = data.clone();
        commands.queue(move |world: &mut World| {
            if let Some(component_editor) = world.remove_resource::<ComponentEditor>() {
                let component_editor =
                    handle_component_update(component_editor, world, entity, &data);
                world.insert_resource(component_editor);
            }
        });
    }
}

fn handle_component_update(
    component_editor: ComponentEditor, // Takes ownership
    world: &mut World,
    entity: Entity,
    data: &EntityRegisteredData,
) -> ComponentEditor {
    // Returns ownership
    if let Some(new_registered) = &data.registered_add_request {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "User requested new component"
        );
        component_editor.add_component_by_name(world, entity, new_registered);
    } else if let Some(delete_request) = &data.registered_remove_request {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "User requested component delete"
        );
        component_editor.remove_component_by_name(world, entity, delete_request);
    } else if data.registered_data_changed {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "User requested component edit - applying changes to {} components",
            data.components.len()
        );
        // Apply all component changes back to the entity
        for component in &data.components {
            component_editor.edit_component_by_name(
                world,
                entity,
                &component.type_name,
                component.reflected_data.as_ref(),
            );
        }
    }
    component_editor // Return it so it can be put back
}

pub fn handle_material_deletion_system(
    mut material_delete_reader: MessageReader<MaterialDeleteEvent>,
    available_materials: Res<AvailableEditableMaterials>,
    mut identity_query: Query<(Entity, &mut IdentityData)>,
    mut commands: Commands,
) {
    for MaterialDeleteEvent { path } in material_delete_reader.read() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Asset,
            "Processing material deletion for path: {}",
            path
        );

        let none_material = if let Some(materials) = &available_materials.materials {
            if !materials.is_empty() {
                materials[0].clone()
            } else {
                log!(
                    LogType::Editor,
                    LogLevel::Error,
                    LogCategory::Asset,
                    "No materials available, cannot reset entities using deleted material"
                );
                continue;
            }
        } else {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Asset,
                "No materials list available, cannot reset entities using deleted material"
            );
            continue;
        };

        for (entity, mut identity) in identity_query.iter_mut() {
            if let Some(material_data) = identity.class.get_mut_material_data() {
                if material_data.current.path == *path {
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::Asset,
                        "Resetting entity material from '{}' to 'None'",
                        path
                    );

                    *material_data.current = none_material.clone();
                    *material_data.path = none_material.path.clone();

                    commands
                        .entity(entity)
                        .insert(MeshMaterial3d::<StandardMaterial>::default());
                }
            }
        }

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::Asset,
            "Completed material deletion processing for '{}'",
            path
        );
    }
}
