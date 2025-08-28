use super::{IdentityData, TransformData};
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
    io::Write,
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

#[derive(Serialize, Deserialize, Debug)]
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

    // Map entity indices to their actual UUIDs from IdentityData
    let mut entity_uuid_map = std::collections::HashMap::new();
    if let Some(entity_vec) = &entities_data {
        for (entity, identity, _, _) in entity_vec.iter() {
            entity_uuid_map.insert(entity.index(), identity.uuid);
        }
    }

    let entities_to_serialize: Vec<EntitySaveReadyData> = match &entities_data {
        Some(entity_vec) => entity_vec
            .iter()
            .map(|(entity, identity, transform, parent)| {
                let translation = round_vec3(transform.translation);
                let rotation = round_quat(transform.rotation);
                let scale = round_vec3(transform.scale);
                let parent_uuid = parent.and_then(|p| entity_uuid_map.get(&p.index()).copied());
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
