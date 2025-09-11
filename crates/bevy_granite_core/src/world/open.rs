use crate::assets::AvailableEditableMaterials;
use crate::events::{RequestLoadEvent, WorldLoadSuccessEvent};
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
    if let Some(RequestLoadEvent(path)) = world_open_reader.read().next() {
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
                "Open world called: {:?}",
                abs_path
            );
        } else {
            abs_path = path.to_string();
        }

        let Ok(version) =
            crate::entities::serialize::SceneMetadata::extracted_version_from_file(&abs_path)
        else {
            log!(
                LogType::Game,
                LogLevel::Error,
                LogCategory::System,
                "Failed to extract version from scene file: {:?}",
                path
            );
            return;
        };

        commands
            .queue(move |world: &mut World| version.deserialize_entities(world, abs_path.into()));
    }
}
