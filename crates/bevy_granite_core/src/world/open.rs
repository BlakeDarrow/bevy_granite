use crate::events::{RequestLoadEvent, WorldLoadSuccessEvent};
use crate::{absolute_asset_to_rel};
use crate::{assets::AvailableEditableMaterials, entities::deserialize_entities};
use bevy::prelude::*;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

/// Watches for RequestLoadEvent and then deserializes the world from its path
pub fn open_world_reader(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    mut available_materials: ResMut<AvailableEditableMaterials>,
    mut world_open_reader: MessageReader<RequestLoadEvent>,
    mut world_load_success_writer: MessageWriter<WorldLoadSuccessEvent>,
) {
    if let Some(RequestLoadEvent(path, save_settings, translation)) =
        world_open_reader.read().next()
    {
        let rel = absolute_asset_to_rel(path.to_string()).to_string();
        deserialize_entities(
            &asset_server,
            &mut commands,
            &mut materials,
            &mut available_materials,
            meshes,
            rel.clone(),
            save_settings.clone(),
            *translation,
        );

        log!(
            LogType::Game,
            LogLevel::OK,
            LogCategory::System,
            "Loaded world: {:?}",
            &rel
        );

        world_load_success_writer.write(WorldLoadSuccessEvent(rel.clone()));
    }
}
