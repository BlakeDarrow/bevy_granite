use super::{IdentityData, TransformData, SaveSettings};
use crate::{get_current_scene_version, world::WorldState};
use bevy::prelude::{Quat, Vec3};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Write, Read},
    path::Path,
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneMetadata {
    pub format_version: String,
    pub entity_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneData {
    pub metadata: SceneMetadata,
    pub entities: Vec<EntitySaveReadyData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntitySaveReadyData {
    pub identity: IdentityData,
    pub transform: TransformData,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>, // Parent entity UUID, needs to be universal if other worlds are loaded in. Bevy id not good enough

    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<HashMap<String, String>>,
}

// want set order or something? only actually save to disk if things changed. Same with editor toml
pub fn serialize_entities(world_state: WorldState, path: Option<String>) {
    let entities_data = world_state.entity_data;
    let runtime_data_provider = world_state.component_data.unwrap_or_default();

    // Read original file data for PreserveDiskFull entities
    let original_entities = if let Some(ref path_str) = path {
        read_existing_file_data(path_str)
    } else {
        Vec::new()
    };
    
    // Create map of UUID -> original data for quick lookup
    let original_by_uuid: HashMap<Uuid, EntitySaveReadyData> = original_entities
        .into_iter()
        .map(|entity| (entity.identity.uuid, entity))
        .collect();

    // Map entity indices to their actual UUIDs from IdentityData
    let mut entity_uuid_map = std::collections::HashMap::new();
    if let Some(entity_vec) = &entities_data {
        for (entity, identity, _, _, _) in entity_vec.iter() {
            entity_uuid_map.insert(entity.index(), identity.uuid);
        }
    }

    let entities_to_serialize: Vec<EntitySaveReadyData> = match &entities_data {
        Some(entity_vec) => entity_vec
            .iter()
            .map(|(entity, identity, transform, parent, save_as)| {
                let parent_uuid = parent.and_then(|p| entity_uuid_map.get(&p.index()).copied());
                
                match save_as {
                    SaveSettings::Runtime => {
                        // Use current world state
                        let translation = round_vec3(transform.translation);
                        let rotation = round_quat(transform.rotation);
                        let scale = round_vec3(transform.scale);
                        EntitySaveReadyData {
                            identity: identity.clone(),
                            transform: TransformData {
                                position: translation,
                                rotation,
                                scale,
                            },
                            parent: parent_uuid, 
                            components: runtime_data_provider.get(entity).cloned(),
                        }
                    },
                    SaveSettings::PreserveDiskTransform => {
                        // Use current world state for everything except transform, which comes from disk
                        let disk_transform = original_by_uuid.get(&identity.uuid)
                            .map(|original| original.transform.clone())
                            .unwrap_or_else(|| {
                                // Fallback to current transform if original not found
                                log!(
                                    LogType::Game,
                                    LogLevel::Warning,
                                    LogCategory::System,
                                    "PreserveDiskTransform entity {} not found in original file, using current transform",
                                    identity.uuid
                                );
                                let translation = round_vec3(transform.translation);
                                let rotation = round_quat(transform.rotation);
                                let scale = round_vec3(transform.scale);
                                TransformData {
                                    position: translation,
                                    rotation,
                                    scale,
                                }
                            });

                        EntitySaveReadyData {
                            identity: identity.clone(),
                            transform: disk_transform,
                            parent: parent_uuid,
                            components: runtime_data_provider.get(entity).cloned(),
                        }
                    }
                    SaveSettings::PreserveDiskFull => {
                        // Use original file data
                        original_by_uuid.get(&identity.uuid)
                            .cloned()
                            .unwrap_or_else(|| {
                                // Fallback to current world state if original not found
                                log!(
                                    LogType::Game,
                                    LogLevel::Warning,
                                    LogCategory::System,
                                    "PreserveDiskFull entity {} not found in original file, using current state",
                                    identity.uuid
                                );
                                let translation = round_vec3(transform.translation);
                                let rotation = round_quat(transform.rotation);
                                let scale = round_vec3(transform.scale);
                                EntitySaveReadyData {
                                    identity: identity.clone(),
                                    transform: TransformData {
                                        position: translation,
                                        rotation,
                                        scale,
                                    },
                                    parent: parent_uuid, 
                                    components: runtime_data_provider.get(entity).cloned(),
                                }
                            })
                    }
                }
            })
            .collect(),
        None => Vec::new(),
    };

    let pretty_config = PrettyConfig::new()
        .depth_limit(15)
        .separate_tuple_members(false)
        .enumerate_arrays(false)
        .compact_arrays(true)
        .indentor("\t".to_string());

    if let Some(path) = path {
        // Create metadata with version from TOML file
        let metadata = SceneMetadata {
            format_version: get_current_scene_version(),
            entity_count: entities_to_serialize.len(),
        };

        // Wrap entities with metadata
        let scene_data = SceneData {
            metadata,
            entities: entities_to_serialize,
        };

        let serialized_data = to_string_pretty(&scene_data, pretty_config).unwrap();

        // TODO:
        // Compress the data (Encrypt?)
        // (and uncompressor)
        let mut file = {
            // Create parent directories first
            if let Some(parent) = Path::new(&path).parent() {
                fs::create_dir_all(parent).unwrap_or_else(|e| {
                    panic!("Failed to create directories for path{}: {e}", path)
                });
            }

            File::create(&path).unwrap_or_else(|e| panic!("Failed to create file{}: {e}", path))
        };

        file.write_all(serialized_data.as_bytes())
            .expect("Failed to write to file");

        log!(
            LogType::Game,
            LogLevel::OK,
            LogCategory::System,
            "Finished serializing to file: '{}'",
            path
        );
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::Blank,
            "-------------"
        );
    }
}

fn round3(f: f32) -> f32 {
    (f * 1000.0).round() / 1000.0
}

fn round_vec3(v: Vec3) -> Vec3 {
    Vec3::new(round3(v.x), round3(v.y), round3(v.z))
}

fn round_quat(q: Quat) -> Quat {
    Quat::from_xyzw(round3(q.x), round3(q.y), round3(q.z), round3(q.w))
}

/// Read existing file data to get original entity data for PreserveDiskFull entities
fn read_existing_file_data(path: &str) -> Vec<EntitySaveReadyData> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return Vec::new();
    }

    let mut file = match std::fs::File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };

    let mut file_contents = String::new();
    if file.read_to_string(&mut file_contents).is_err() {
        return Vec::new();
    }

    if file_contents.trim().is_empty() {
        return Vec::new();
    }

    // Try to parse with new format first (with metadata), fallback to old format
    if let Ok(scene_data) = ron::de::from_str::<SceneData>(&file_contents) {
        scene_data.entities
    } else if let Ok(entities) = ron::de::from_str::<Vec<EntitySaveReadyData>>(&file_contents) {
        entities
    } else {
        log!(
            LogType::Game,
            LogLevel::Warning,
            LogCategory::System,
            "Failed to parse existing file data from: {}",
            path
        );
        Vec::new()
    }
}
