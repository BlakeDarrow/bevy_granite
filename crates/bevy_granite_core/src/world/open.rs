use crate::events::{RequestLoadEvent, WorldLoadSuccessEvent};
use crate::{assets::AvailableEditableMaterials, entities::deserialize_entities};
use bevy::{asset::io::file::FileAssetReader, prelude::*};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use std::path::Path;

/// Watches for RequestLoadEvent and then deserializes the world from its path
pub fn open_world_reader(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    meshes: ResMut<Assets<Mesh>>,
    mut available_materials: ResMut<AvailableEditableMaterials>,
    mut world_open_reader: EventReader<RequestLoadEvent>,
    mut world_load_success_writer: EventWriter<WorldLoadSuccessEvent>,
) {
    if let Some(RequestLoadEvent(path, save_settings, translation)) = world_open_reader.read().next() {
        let abs_path: String;
        if !Path::new(path).is_absolute() {
            abs_path = FileAssetReader::get_base_path()
                .join("assets/".to_string() + path)
                .to_string_lossy()
                .to_string();
            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::System,
                "Open world called: '{:?}' with offset: '{:?}'",
                abs_path,
                translation
            );
        } else {
            abs_path = path.to_string();
        }

        deserialize_entities(
            &asset_server,
            &mut commands,
            &mut materials,
            &mut available_materials,
            meshes,
            abs_path,
            save_settings.clone(),
            *translation
        );

        log!(
            LogType::Game,
            LogLevel::OK,
            LogCategory::System,
            "Loaded world: {:?}",
            path
        );

        world_load_success_writer.write(WorldLoadSuccessEvent(path.to_string()));
    }
}
