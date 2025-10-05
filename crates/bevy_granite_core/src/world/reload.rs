use crate::{
    absolute_asset_to_rel,
    entities::{despawn_recursive_serializable_entities, IdentityData, SaveSettings},
    events::{RequestLoadEvent, RequestReloadEvent},
};
use bevy::prelude::{Commands, Entity, MessageReader, MessageWriter, Query, With};

/// Despawns all entities then loads the world
pub fn reload_world_system(
    mut relead_watcher: MessageReader<RequestReloadEvent>,
    mut commands: Commands,
    serializable_query: Query<Entity, With<IdentityData>>,
    mut load_world_writter: MessageWriter<RequestLoadEvent>,
) {
    for RequestReloadEvent(path) in relead_watcher.read() {
        despawn_recursive_serializable_entities(&mut commands, &serializable_query);
        // need to have better way to do undo... actually use events
        load_world_writter.write(RequestLoadEvent(
            absolute_asset_to_rel(path.to_string()).to_string(),
            SaveSettings::Runtime,
            None,
        ));
    }
}
