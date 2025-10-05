use super::{IdentityData, SpawnSource};
use crate::events::{RequestDespawnBySource, RequestDespawnSerializableEntities};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use bevy::prelude::{Commands, Entity, MessageReader, Query, With};

/// If entity has IdentityData, it is despawned
pub fn despawn_entities_system(
    mut despawn_watcher: MessageReader<RequestDespawnSerializableEntities>,
    mut commands: Commands,
    serializable_query: Query<Entity, With<IdentityData>>,
) {
    for RequestDespawnSerializableEntities in despawn_watcher.read() {
        despawn_recursive_serializable_entities(&mut commands, &serializable_query);

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::System,
            "Despawned serializable entities"
        );
    }
}

/// Despawn entities that have a specific SpawnSource
pub fn despawn_entities_by_source_system(
    mut despawn_watcher: MessageReader<RequestDespawnBySource>,
    mut commands: Commands,
    serializable_query: Query<(Entity, &SpawnSource), With<IdentityData>>,
) {
    for RequestDespawnBySource(source) in despawn_watcher.read() {
        let mut despawned_count = 0;

        for (entity, entity_source) in serializable_query.iter() {
            if entity_source.0 == *source {
                commands.entity(entity).try_despawn();
                despawned_count += 1;
            }
        }

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::System,
            "Despawned {} entities from source: '{}'",
            despawned_count,
            source
        );
    }
}

// Despawn recursive
pub fn despawn_recursive_serializable_entities(
    commands: &mut Commands,
    serializable_query: &Query<Entity, With<IdentityData>>,
) {
    for entity in serializable_query.iter() {
        commands.entity(entity).try_despawn();
    }
}
