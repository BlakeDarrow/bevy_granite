use crate::{
    editor_state::{get_dock_state_str, EditorState},
    interface::{BottomDockState, EditorSettingsTabData, SetActiveWorld, SideDockState},
};

use crate::utils::{load_from_toml_file, save_to_toml_file};
use bevy::ecs::message::MessageReader;
use bevy::{asset::io::file::FileAssetReader, prelude::ResMut};
use bevy_granite_core::{
    absolute_asset_to_rel,
    events::{
        RequestDespawnBySource, RequestDespawnSerializableEntities, WorldLoadSuccessEvent,
        WorldSaveSuccessEvent,
    },
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use crate::interface::RequestEditorToggle;
use bevy::prelude::Commands;
use bevy_granite_gizmos::{selection::events::EntityEvents, GizmoVisibilityState};

// editor.rs
// This has functions related to saving the editor settings
// Currently the settings data is coming directly from the right Tab settings.

pub fn update_active_world_system(
    mut open_success_reader: MessageReader<WorldLoadSuccessEvent>,
    mut world_save_success_reader: MessageReader<WorldSaveSuccessEvent>,
    mut entities_despawned_reader: MessageReader<RequestDespawnSerializableEntities>,
    mut entities_despawned_by_source_reader: MessageReader<RequestDespawnBySource>,
    mut set_active_world_reader: MessageReader<SetActiveWorld>,
    mut editor_state: ResMut<EditorState>,
) {
    for RequestDespawnSerializableEntities in entities_despawned_reader.read() {
        editor_state.current_file = None;
        editor_state.loaded_sources.clear();
        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::System,
            "Despawned all serializable entities and cleared loaded sources"
        );
    }

    for RequestDespawnBySource(source) in entities_despawned_by_source_reader.read() {
        editor_state.loaded_sources.remove(source);

        // If the current file was despawned, clear it
        if editor_state.current_file.as_ref() == Some(source) {
            editor_state.current_file = None;
        }

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::System,
            "Removed source '{}' from loaded sources list",
            source
        );
    }

    for WorldLoadSuccessEvent(path) in open_success_reader.read() {
        let rel_path = absolute_asset_to_rel(path.to_string());
        editor_state.current_file = Some(rel_path.to_string());
        editor_state.loaded_sources.insert(rel_path.to_string());
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "Added source '{}' to loaded sources list (total: {})",
            rel_path,
            editor_state.loaded_sources.len()
        );
    }

    for WorldSaveSuccessEvent(path) in world_save_success_reader.read() {
        let rel_path = absolute_asset_to_rel(path.to_string());
        editor_state.loaded_sources.insert(rel_path.to_string());
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "World saved '{}'. Loaded sources list (total: {})",
            rel_path,
            editor_state.loaded_sources.len()
        );
    }

    for SetActiveWorld(path) in set_active_world_reader.read() {
        let rel_path = absolute_asset_to_rel(path.to_string());
        editor_state.current_file = Some(rel_path.to_string());
        editor_state.loaded_sources.insert(rel_path.to_string());
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "World '{}' set as active. Loaded sources list (total: {})",
            rel_path,
            editor_state.loaded_sources.len()
        );
    }
}

pub fn update_editor_vis_system(
    mut toggle_reader: MessageReader<RequestEditorToggle>,
    mut editor_state: ResMut<EditorState>,
    mut gizmo_state: ResMut<GizmoVisibilityState>,
    mut commands: Commands,
) {
    for RequestEditorToggle in toggle_reader.read() {
        // have to do it manually as watchers wont run when not active
        commands.trigger(EntityEvents::DeselectAll);

        editor_state.active = !editor_state.active;
        gizmo_state.active = editor_state.active;
    }
}

// FIX: Probably need to decouple this from the UI tab. Sep res for editor data and keep them synced.
pub fn save_editor_settings_from_widget_data(
    editor_state: &mut ResMut<EditorState>,
    editor_settings: &mut EditorSettingsTabData,
    right_dock: SideDockState,
    bottom_dock: BottomDockState,
) {
    let config_path_buf =
        FileAssetReader::get_base_path().join("assets/".to_string() + &editor_state.config_path);

    editor_settings.dock.layout_str = get_dock_state_str(right_dock, bottom_dock);

    if let Some(config_path_str) = config_path_buf.to_str() {
        match save_to_toml_file(editor_settings, config_path_str) {
            Ok(_editor_config_content) => {
                editor_state.config = editor_settings.clone();

                log!(
                    LogType::Editor,
                    LogLevel::OK,
                    LogCategory::System,
                    "Saved editor config toml",
                );
            }
            Err(e) => {
                log!(
                    LogType::Editor,
                    LogLevel::Error,
                    LogCategory::System,
                    "Failed to save editor config: {} - {}",
                    e,
                    config_path_str
                );
            }
        }
    } else {
        log!(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::System,
            "Invalid UTF-8 path for config: {:?}",
            config_path_buf
        );
    }
}

/// Generic function to update any field in the editor TOML configuration
/// Takes a closure that modifies the config and saves it automatically
pub fn update_editor_config_field<F>(
    editor_state: &mut ResMut<EditorState>,
    update_fn: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut EditorSettingsTabData),
{
    let config_path_buf =
        FileAssetReader::get_base_path().join("assets/".to_string() + &editor_state.config_path);

    if let Some(config_path_str) = config_path_buf.to_str() {
        // Apply the update function to modify the config
        update_fn(&mut editor_state.config);

        // Save the updated config
        match save_to_toml_file(&editor_state.config, config_path_str) {
            Ok(_) => {
                log!(
                    LogType::Editor,
                    LogLevel::OK,
                    LogCategory::System,
                    "Updated and saved editor config field",
                );
                Ok(())
            }
            Err(e) => {
                log!(
                    LogType::Editor,
                    LogLevel::Error,
                    LogCategory::System,
                    "Failed to save updated editor config: {} - {}",
                    e,
                    config_path_str
                );
                Err(e.into())
            }
        }
    } else {
        let error_msg = format!("Invalid UTF-8 path for config: {:?}", config_path_buf);
        log!(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::System,
            "{}",
            error_msg
        );
        Err(error_msg.into())
    }
}

pub fn load_editor_settings_toml(mut editor_state: ResMut<EditorState>) {
    let config_path_buf =
        FileAssetReader::get_base_path().join("assets/".to_string() + &editor_state.config_path);
    if let Some(config_path_str) = config_path_buf.to_str() {
        match load_from_toml_file(config_path_str) {
            Ok(editor_config_content) => {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::System,
                    "Loaded editor config toml",
                );
                editor_state.config = editor_config_content;
                editor_state.config_loaded = true;
            }
            Err(e) => {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::System,
                    "Failed to load editor config: {} - {}. Using defaults and attempting to save updated config.",
                    e,
                    config_path_str
                );

                // Use default config when loading fails
                editor_state.config = EditorSettingsTabData::default();
                editor_state.config_loaded = true;

                // Attempt to save the default config to fix the file
                if let Err(save_err) = save_to_toml_file(&editor_state.config, config_path_str) {
                    log!(
                        LogType::Editor,
                        LogLevel::Error,
                        LogCategory::System,
                        "Failed to save default config: {}",
                        save_err
                    );
                } else {
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::System,
                        "Successfully saved default config to {}",
                        config_path_str
                    );
                }
            }
        }
    } else {
        log!(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::System,
            "Invalid UTF-8 path for config: {:?}",
            config_path_buf
        );
    }
}
