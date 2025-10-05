use crate::{
    entities::{serialize_entities, ComponentEditor, HasRuntimeData, IdentityData, SpawnSource},
    events::{CollectRuntimeDataEvent, RequestSaveEvent, RuntimeDataReadyEvent},
    shared::absolute_asset_to_rel,
    WorldSaveSuccessEvent,
};
use bevy::{
    asset::io::file::FileAssetReader,
    ecs::entity::Entity,
    prelude::{ChildOf, Commands, MessageReader, MessageWriter, Query, ResMut, Resource, World},
    transform::components::Transform,
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use std::path::PathBuf;
use std::{borrow::Cow, collections::HashMap};

#[derive(Default, Debug, Clone)]
pub struct WorldState {
    // Can easily be queried for, so we can immediately get this data
    pub entity_data: Option<Vec<(Entity, IdentityData, Transform, Option<Entity>, crate::entities::SaveSettings)>>, // Added parent entity, UUID, and SaveSettings

    // More difficult to get, so we do no have this off rip
    // We need to use World and the type registry to build and send event back saying its ready
    pub component_data: Option<HashMap<Entity, HashMap<String, String>>>,

    // Inside world runner, when gathered this flag gets set
    pub components_ready: bool,
}

#[derive(Resource, Default)]
pub struct SaveWorldRequestData {
    pub pending_saves: HashMap<Cow<'static, str>, (PathBuf, WorldState)>, // source -> (path, world_state)
}

/// Part 1.
/// We gather all entities that are serializeable with
/// IdentityData and Transform
///
/// Part 2.
/// Is runtime collector for registered type components
pub fn save_request_system(
    mut save_request: ResMut<SaveWorldRequestData>,
    mut event_writer: MessageWriter<CollectRuntimeDataEvent>,
    mut event_reader: MessageReader<RequestSaveEvent>,
    query: Query<(
        Entity,
        &IdentityData,
        Option<&Transform>,
        Option<&ChildOf>,
        &SpawnSource,
    )>,
) {
    // Process only one save request per frame to avoid conflicts
    if let Some(RequestSaveEvent(path)) = event_reader.read().next() {
        let spawn_source = absolute_asset_to_rel(path.clone());

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "Save request for source: '{}' (from path: '{}')",
            spawn_source,
            path
        );

        event_writer.write(CollectRuntimeDataEvent(spawn_source.to_string()));

        // Part 1.
        // Gather all entities that are serializeable and contain IdentityData and Transform
        // Filter by SpawnSource to only include entities from the target source
        let entities_data: Vec<(Entity, IdentityData, Transform, Option<Entity>, crate::entities::SaveSettings)> = query
            .iter()
            .filter(|(_, _, _, _, source)| source.str_ref() == spawn_source)
            .map(|(entity, obj, transform, relation, source)| {
                (
                    entity,
                    obj.clone(),
                    transform.cloned().unwrap_or_default(),
                    relation.map(|r| r.parent()),
                    source.save_settings_ref().clone(),
                )
            })
            .collect();

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "Found {} entities with SpawnSource '{}'",
            entities_data.len(),
            spawn_source
        );

        let asset_path = FileAssetReader::get_base_path()
            .join("assets")
            .join(path.clone());

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "Want to save world at: {:?}",
            asset_path.display()
        );

        // Step 2.
        // We need to gather components
        // so we set a pending save for another system to fill in
        let world_state = WorldState {
            entity_data: Some(entities_data),
            component_data: None,
            components_ready: false,
        };

        save_request
            .pending_saves
            .insert(spawn_source.clone(), (asset_path.clone(), world_state));

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "Save request: {:?}",
            asset_path
        );
    }
}

/// This is part 2 of world save request.
/// We gather components
/// Simplified version using the new consolidated patterns
pub fn collect_components_system(
    mut commands: Commands,
    runtime_query: Query<(Entity, &HasRuntimeData, &SpawnSource)>,
    mut event_reader: MessageReader<CollectRuntimeDataEvent>,
) {
    for CollectRuntimeDataEvent(spawn_source) in event_reader.read() {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::Blank,
            "Collecting components for source: {}",
            spawn_source
        );

        // Filter entities to only include those with matching SpawnSource
        let entities: Vec<Entity> = runtime_query
            .iter()
            .filter(|(_, _, source)| source.str_ref() == *spawn_source)
            .map(|(entity, _, _)| entity)
            .collect();

        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::Entity,
            "Found {} entities with HasRuntimeData and SpawnSource '{}'",
            entities.len(),
            spawn_source
        );

        // Clone spawn_source for the closure
        let spawn_source_clone: Cow<'static, str> = spawn_source.clone().into();

        // Need access to world to get components
        commands.queue(move |world: &mut World| {
            let component_editor = world.resource::<ComponentEditor>();
            let mut collected_data = HashMap::new();

            for entity in entities {
                let serialized_components =
                    component_editor.serialize_entity_components(world, entity);

                if !serialized_components.is_empty() {
                    collected_data.insert(entity, serialized_components);
                }
            }

            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::Entity,
                "Collected components: {:?}",
                collected_data
            );

            if let Some(mut data) = world.get_resource_mut::<SaveWorldRequestData>() {
                if let Some((_, world_state)) = data.pending_saves.get_mut(&spawn_source_clone) {
                    world_state.component_data = Some(collected_data);
                    world_state.components_ready = true;

                    log!(
                        LogType::Game,
                        LogLevel::Info,
                        LogCategory::System,
                        "Sending RuntimeDataReadyEvent for source: {}",
                        spawn_source_clone
                    );

                    world.send_event(RuntimeDataReadyEvent(spawn_source_clone.to_string()));
                }
            }
        });
    }
}

/// Component data is ready, we can save the world
pub fn save_data_ready_system(
    mut event_reader: MessageReader<RuntimeDataReadyEvent>,
    mut save_request_data: ResMut<SaveWorldRequestData>,
    mut saved_event_writer: MessageWriter<WorldSaveSuccessEvent>,
) {
    for RuntimeDataReadyEvent(source) in event_reader.read() {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "Save data ready for source: {}",
            source
        );
        let source: &str = source.as_ref();

        if let Some((path, world_state)) = save_request_data.pending_saves.remove(source) {
            if !world_state.components_ready {
                log!(
                    LogType::Game,
                    LogLevel::Critical,
                    LogCategory::System,
                    "Runtime component gathering failed for source '{}' - Will not serialize",
                    source
                );
                continue;
            }

            log!(
                LogType::Game,
                LogLevel::OK,
                LogCategory::System,
                "Components gathered and ready to save for source '{}'",
                source
            );
            serialize_entities(world_state, Some(path.display().to_string()));
            log!(
                LogType::Game,
                LogLevel::OK,
                LogCategory::System,
                "Saved world: {:?}",
                path
            );
            saved_event_writer.write(WorldSaveSuccessEvent(path.display().to_string()));
        } else {
            log!(
                LogType::Game,
                LogLevel::Error,
                LogCategory::System,
                "No pending save found for source: '{}'",
                source
            );
        }
    }
}
