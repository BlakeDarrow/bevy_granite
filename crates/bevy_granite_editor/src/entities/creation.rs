use crate::{editor_state::EditorState, interface::UserRequestGraniteTypeViaPopup};
use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        message::MessageReader,
        system::{Commands, Res, ResMut},
    },
    mesh::Mesh,
    pbr::StandardMaterial,
    prelude::Resource,
    transform::components::Transform,
};
use bevy_granite_core::{
    entities::{GraniteType, SaveSettings, SpawnSource},
    shared::asset_file_browser_multiple,
    AvailableEditableMaterials, GraniteTypes, PromptData, PromptImportSettings,
};
use bevy_granite_gizmos::selection::events::EntityEvents;
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};
use std::collections::VecDeque;

#[derive(Resource, Default)]
pub struct EntitySpawnQueue {
    pub pending: VecDeque<PendingEntitySpawn>,
    pub current_batch_size: usize,
}

#[derive(Clone, PartialEq)]
pub struct PendingEntitySpawn {
    pub class: GraniteTypes,
    pub file: Option<String>,
    pub transform: Transform,
    pub source: String,
    pub batch_size: usize,
}

// Popup to queues entity spawns. Handles single and multiple
pub fn new_entity_via_popup_system(
    mut entity_add_reader: MessageReader<UserRequestGraniteTypeViaPopup>,
    mut spawn_queue: ResMut<EntitySpawnQueue>,
    editor_state: Res<EditorState>,
    mut commands: Commands,
) {
    if let Some(UserRequestGraniteTypeViaPopup { class }) = entity_add_reader.read().next() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "User wants to add via popup: {:?}",
            class
        );

        let transform = Transform::default();

        let source = editor_state
            .current_file
            .clone()
            .unwrap_or_else(|| "user".to_string());

        if class.needs_prompt() {
            let (base_dir, filter) = class.get_prompt_config();
            if let Some(files) = asset_file_browser_multiple(base_dir, filter) {
                let batch_size = files.len();
                if batch_size > 1 {
                    commands.trigger(EntityEvents::DeselectAll);
                }
                spawn_queue.current_batch_size = batch_size;

                // Queue each file as a separate spawn
                for file in files {
                    spawn_queue.pending.push_back(PendingEntitySpawn {
                        class: class.clone(),
                        file: Some(file),
                        transform,
                        source: source.clone(),
                        batch_size,
                    });
                }
            }
        } else {
            let batch_size = 1;
            spawn_queue.current_batch_size = batch_size;

            // Queue single spawn without file
            spawn_queue.pending.push_back(PendingEntitySpawn {
                class: class.clone(),
                file: None,
                transform,
                source: source.clone(),
                batch_size,
            });
        }

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Queued {} entities for spawning",
            spawn_queue.pending.len()
        );
    }
}

pub fn process_entity_spawn_queue_system(
    mut spawn_queue: ResMut<EntitySpawnQueue>,
    mut commands: Commands,
    available_materials: ResMut<AvailableEditableMaterials>,
    standard_materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    prompt_settings: Res<PromptImportSettings>,
) {
    if let Some(mut pending) = spawn_queue.pending.pop_front() {
        let prompt_data = PromptData {
            file: pending.file,
            import_settings: prompt_settings.clone(),
        };

        let entity = pending.class.spawn_from_new_identity(
            &mut commands,
            pending.transform,
            standard_materials,
            meshes,
            available_materials,
            asset_server,
            Some(prompt_data),
        );

        // Tag entity with spawn source
        commands.entity(entity).insert(SpawnSource::new(
            pending.source.clone(),
            SaveSettings::Runtime,
        ));

        let additive = pending.batch_size > 1;
        let remaining = spawn_queue.pending.len();

        commands.trigger(EntityEvents::Select {
            target: entity,
            additive,
        });

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::Entity,
            "Spawned entity: '{:?}' from queue, {} remaining",
            entity,
            remaining
        );
    }
}
