use crate::{
    entities::{despawn_recursive_serializable_entities, SaveSettings, IdentityData},
    events::{RequestLoadEvent, RequestReloadEvent},
};
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, Query, With};

/// Despawns all entities then loads the world
pub fn reload_world_system(
    mut relead_watcher: EventReader<RequestReloadEvent>,
    mut commands: Commands,
    serializable_query: Query<Entity, With<IdentityData>>,
    mut load_world_writter: EventWriter<RequestLoadEvent>,
) {
    for RequestReloadEvent(path) in relead_watcher.read() {
        despawn_recursive_serializable_entities(&mut commands, &serializable_query);
        // need to have better way to do undo... actually use events
        load_world_writter.write(RequestLoadEvent(
            path.to_string(),
            SaveSettings::Runtime,
            None,
        ));
    }
}
