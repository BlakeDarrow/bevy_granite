use super::{ComponentEditor, EntitySaveReadyData, IdentityData, SceneData, SpawnSource};
use crate::{
    absolute_asset_to_rel, entities::SaveSettings, materials_from_folder_into_scene,
    rel_asset_to_absolute, shared::is_scene_version_compatible, AvailableEditableMaterials,
    GraniteType, TransformData,
};
use bevy::{
    ecs::{entity::Entity, system::ResMut, world::World},
    mesh::Mesh,
    pbr::StandardMaterial,
    prelude::{AppTypeRegistry, AssetServer, Assets, Commands, Component, Reflect, Res},
    transform::components::Transform,
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use ron::de::from_str;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::{borrow::Cow, fs::File};
use uuid::Uuid;

// Main component to tag all of our custom entity class types
// if its spawned through granite_core it should have this component
#[derive(Component, Serialize, Reflect, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct GraniteEditorSerdeEntity;

// Basically we grab the file contents into save ready struct
// Spawn all entities - (might be able to improve and just insert components this step?)
// Insert all components with access to mut World after all entities are spawned

/// Build materials and entities into the scene from the world path
pub fn deserialize_entities(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    available_materials: &mut ResMut<AvailableEditableMaterials>,
    meshes: &mut ResMut<Assets<Mesh>>,
    path: impl Into<Cow<'static, str>>, //absolute or rel
    save_settings: SaveSettings,
    transform_override: Option<Transform>,
) {
    let abs_path: Cow<'static, str> = rel_asset_to_absolute(&path.into());
    // Build materials from the folder and load them into the scene
    materials_from_folder_into_scene("materials", materials, available_materials, asset_server);

    // Gather file contents into a Vec<EntitySaveReadyData>
    let deserialized_data = gather_file_contents(
        asset_server,
        materials,
        available_materials,
        abs_path.as_ref(),
    );

    // for id
    let mut uuid_to_entity_map: std::collections::HashMap<Uuid, Entity> =
        std::collections::HashMap::new();
    let mut parent_relationships: Vec<(Entity, Uuid)> = Vec::new(); // (child_entity, parent_guid)

    // Deserialized data is Vec<EntitySaveReadyData>
    for save_data in &deserialized_data {
        let (entity, _final_identity) = spawn_entity_from_class_type(
            asset_server,
            commands,
            materials,
            available_materials,
            meshes,
            save_data,
            transform_override,
        );

        // Map the stored GUID to the new entity
        uuid_to_entity_map.insert(save_data.identity.uuid, entity);

        // Tag entity with its source file
        let relative: Cow<'static, str> = absolute_asset_to_rel(abs_path.to_string());
        commands
            .entity(entity)
            .insert(SpawnSource::new(relative, save_settings.clone()));

        // Store parent relationships for second pass
        if let Some(parent_guid) = save_data.parent {
            parent_relationships.push((entity, parent_guid));
        }

        //
        //log!(
        //    LogType::Game,
        //    LogLevel::Info,
        //    LogCategory::Entity,
        //    "Inserted: {:?}",
        //    final_identity
        //);

        //log!(
        //    LogType::Game,
        //    LogLevel::Info,
        //    LogCategory::Entity,
        //    "Found Components {:?}",
        //    save_data.components,
        //);
        //

        // Load components into the scene entities
        if let Some(component_map) = save_data.components.as_ref() {
            let component_map = component_map.clone();
            let entity_copy = entity;

            commands.queue(move |world: &mut World| {
                // Get the current type registry from the world
                let type_registry = world.resource::<AppTypeRegistry>().clone();

                // Remove the resource to avoid borrowing errors
                if let Some(component_editor) = world.remove_resource::<ComponentEditor>() {
                    component_editor.load_components_from_scene_data(
                        world,
                        entity_copy,
                        component_map,
                        type_registry,
                    );

                    world.insert_resource(component_editor);
                }
            });
        }
    }

    // Apply relationships
    for (child_entity, parent_guid) in parent_relationships {
        if let Some(&parent_entity) = uuid_to_entity_map.get(&parent_guid) {
            commands.entity(parent_entity).add_child(child_entity);
        } else {
            log!(
                LogType::Game,
                LogLevel::Warning,
                LogCategory::Entity,
                "Could not find parent entity with GUID {} for child {:?}",
                parent_guid,
                child_entity
            );
        }
    }

    log!(
        LogType::Game,
        LogLevel::OK,
        LogCategory::System,
        "Deserialization done"
    );
    log!(
        LogType::Game,
        LogLevel::Info,
        LogCategory::Blank,
        "--------------------"
    );
}

/// Gathers the file contents from the given path and deserializes them into EntitySaveReadyData
fn gather_file_contents(
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    available_materials: &mut ResMut<AvailableEditableMaterials>,
    path: &str,
) -> Vec<EntitySaveReadyData> {
    log!(
        LogType::Game,
        LogLevel::Info,
        LogCategory::Blank,
        "--------------------"
    );
    log!(
        LogType::Game,
        LogLevel::Info,
        LogCategory::Blank,
        "Deserialize Entities"
    );
    log!(
        LogType::Game,
        LogLevel::Info,
        LogCategory::Blank,
        "--------------------"
    );

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            log!(
                LogType::Game,
                LogLevel::Error,
                LogCategory::System,
                "Failed to open file {}: {}. Are you sure it exists?",
                path,
                e
            );
            return vec![];
        }
    };

    let mut file_contents = String::new();
    if let Err(e) = file.read_to_string(&mut file_contents) {
        log!(
            LogType::Game,
            LogLevel::Error,
            LogCategory::System,
            "Failed to read file {}: {}",
            path,
            e
        );
        return vec![];
    }

    // Handle empty file
    if file_contents.is_empty() {
        log!(
            LogType::Game,
            LogLevel::Warning,
            LogCategory::System,
            "No contents found in scene: {}",
            path
        );
        return vec![];
    }

    // Handle whitespace-only files
    if file_contents.trim().is_empty() {
        log!(
            LogType::Game,
            LogLevel::Warning,
            LogCategory::System,
            "Only whitespace found in scene: {}",
            path
        );
        return vec![];
    }

    // Handle empty JSON object or array
    let trimmed = file_contents.trim();
    if trimmed == "{}" || trimmed == "[]" {
        log!(
            LogType::Game,
            LogLevel::Warning,
            LogCategory::System,
            "Empty JSON structure found in scene: {}, skipping entity creation",
            path
        );
        // Still create materials even if no entities to deserialize
        materials_from_folder_into_scene("materials", materials, available_materials, asset_server);
        return vec![];
    }

    // Attempt to deserialize with proper error handling
    // Try new format first (with metadata), fallback to old format (direct array)
    let deserialized_data: Vec<EntitySaveReadyData> = if let Ok(scene_data) =
        from_str::<SceneData>(&file_contents)
    {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "Loading scene with metadata - Version: {}, Entities: {}",
            scene_data.metadata.format_version,
            scene_data.metadata.entity_count
        );

        // Check version compatibility
        if !is_scene_version_compatible(scene_data.metadata.format_version) {
            log!(
                LogType::Game,
                LogLevel::Warning,
                LogCategory::System,
                "Scene version {} may not be fully compatible with current version",
                scene_data.metadata.format_version
            );
        }

        let e_count = scene_data.entities.len();
        if e_count != scene_data.metadata.entity_count {
            log!(
                    LogType::Game,
                    LogLevel::Warning,
                    LogCategory::System,
                    "Entity count mismatch: expected {}, found {}. Not an error, but perhaps you manually edited the scene file?",
                    scene_data.metadata.entity_count,
                    e_count
                );
        }

        scene_data.entities
    } else {
        log!(
            LogType::Game,
            LogLevel::Error,
            LogCategory::System,
            "Failed to deserialize data from {} - invalid format",
            path
        );
        return vec![];
    };

    // Handle case where deserialization succeeded but resulted in empty vector
    if deserialized_data.is_empty() {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "No entities found in scene: {}",
            path
        );
        // Still create materials even if no entities
        materials_from_folder_into_scene("materials", materials, available_materials, asset_server);
        return vec![];
    }

    log!(
        LogType::Game,
        LogLevel::Info,
        LogCategory::System,
        "Successfully loaded {} entities from scene: {}",
        deserialized_data.len(),
        path
    );
    deserialized_data
}

/// Spawns the entity and returns the identity data and entity
fn spawn_entity_from_class_type(
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    available_materials: &mut ResMut<AvailableEditableMaterials>,
    meshes: &mut ResMut<Assets<Mesh>>,
    save_data: &EntitySaveReadyData,
    //transform_override: Option<Transform>,
    transform_offset: Option<Transform>,
) -> (Entity, IdentityData) {
    let class = save_data.identity.class.clone();
    let mut modified_save_data = save_data.clone();

    // Full override
    // Apply transform override if provided and parent entity
    //if let Some(transform_override) = transform_override {
    //    if save_data.parent.is_none() {
    //        modified_save_data.transform = TransformData {
    //            position: transform_override.translation,
    //            rotation: transform_override.rotation,
    //            scale: transform_override.scale,
    //        };
    //    }
    //}

    // Apply transform offset if provided and parent entity
    if let Some(offset) = transform_offset {
        if save_data.parent.is_none() {
            modified_save_data.transform =
                offset_saved_transform(modified_save_data.transform, offset);
        }
    }

    let entity = class.spawn_from_save_data(
        &modified_save_data,
        commands,
        materials,
        meshes,
        available_materials,
        asset_server,
    );

    (entity, save_data.identity.clone())
}

fn offset_saved_transform(original: TransformData, offset: Transform) -> TransformData {
    let original_transform = original.to_bevy();
    let new_transform = offset.mul_transform(original_transform);

    TransformData {
        position: new_transform.translation,
        rotation: new_transform.rotation,
        scale: new_transform.scale,
    }
}
