use crate::assets::SceneAsset;
use crate::events::{
    RequestLoadBatchEvent, RequestLoadEvent, WorldLoadBatchSuccessEvent, WorldLoadSuccessEvent,
};
use crate::{absolute_asset_to_rel, StringAsset, PreloadedMaterialHandles};
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut available_materials: ResMut<AvailableEditableMaterials>,
    scene_assets: Res<Assets<SceneAsset>>,
    string_assets: Res<Assets<StringAsset>>,
    preloaded_materials: Res<PreloadedMaterialHandles>,
    mut world_open_reader: MessageReader<RequestLoadEvent>,
    mut world_load_success_writer: MessageWriter<WorldLoadSuccessEvent>,
) {
    for RequestLoadEvent(path, save_settings, translation) in world_open_reader.read() {
        let rel = absolute_asset_to_rel(path.to_string()).to_string();
        deserialize_entities(
            &asset_server,
            &mut commands,
            &mut materials,
            &mut available_materials,
            &mut meshes,
            &scene_assets,
            &string_assets,
            &preloaded_materials,
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

/// Watches for RequestLoadBatchEvent and then deserializes all worlds from their paths
pub fn open_world_batch_reader(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut available_materials: ResMut<AvailableEditableMaterials>,
    scene_assets: Res<Assets<SceneAsset>>,
    string_assets: Res<Assets<StringAsset>>,
    preloaded_materials: Res<PreloadedMaterialHandles>,
    mut world_batch_reader: MessageReader<RequestLoadBatchEvent>,
    mut world_load_batch_success_writer: MessageWriter<WorldLoadBatchSuccessEvent>,
) {
    for RequestLoadBatchEvent(worlds) in world_batch_reader.read() {
        let mut loaded_paths = Vec::new();

        for (path, save_settings, translation) in worlds.iter() {
            let rel = absolute_asset_to_rel(path.to_string()).to_string();
            deserialize_entities(
                &asset_server,
                &mut commands,
                &mut materials,
                &mut available_materials,
                &mut meshes,
                &scene_assets,
                &string_assets,
                &preloaded_materials,
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

            loaded_paths.push(rel);
        }

        log!(
            LogType::Game,
            LogLevel::OK,
            LogCategory::System,
            "Batch load completed: {} worlds loaded",
            loaded_paths.len()
        );

        world_load_batch_success_writer.write(WorldLoadBatchSuccessEvent(loaded_paths));
    }
}
