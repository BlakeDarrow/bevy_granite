use crate::{
    editor_state::EditorState, interface::{
        BottomDockState, EditorSettingsTabData, SideDockState, SideTab
    }
};
use bevy::{asset::io::file::FileAssetReader, prelude::{MessageReader, Res}};
use bevy::window::WindowClosing;
use crate::utils::{load_from_toml_file, save_to_toml_file};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use toml::{from_str, to_string};

// dock.rs
// This has functions related to saving and loading of the egui dock layout
// We directly serialize the SideDockState and BottomDockState, excluding the actual contained
// data, leaving just the egui state

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct DockLayoutStr {
    pub right_dock_state: Option<String>,
    pub right_dock_width: Option<f32>,
    pub bottom_dock_state: Option<String>,
    pub bottom_dock_height: Option<f32>,
}

pub fn save_dock_on_window_close_system(
    mut window_close_events: MessageReader<WindowClosing>,
    editor_state: Res<EditorState>,
    side_dock_res: Res<SideDockState>,
    bottom_dock_res: Res<BottomDockState>,
) {
    for _event in window_close_events.read() {
        save_dock_layout_toml(
            editor_state.deref().clone(),
            side_dock_res.clone(),
            bottom_dock_res.clone(),
        );
    }
}

pub fn get_dock_state_str(
    right_dock_state: SideDockState,
    bottom_dock_state: BottomDockState,
) -> DockLayoutStr {
    let right_tree = to_string(&right_dock_state.dock_state).unwrap();
    let right_width = right_dock_state.width;
    let bottom_tree = to_string(&bottom_dock_state.dock_state).unwrap();
    let bottom_height = bottom_dock_state.height;

    DockLayoutStr {
        right_dock_state: Some(right_tree),
        right_dock_width: right_width,
        bottom_dock_state: Some(bottom_tree),
        bottom_dock_height: bottom_height,
    }
}

pub fn load_dock_state(
    dock_layout: &DockLayoutStr,
    right_dock_state: &mut SideDockState,
    bottom_dock_state: &mut BottomDockState,
) {
    if let Some(ref right_tree) = dock_layout.right_dock_state {
        if let Ok(dock_state) = from_str(right_tree) {
            right_dock_state.dock_state = dock_state;
        }
    }

    right_dock_state.width = dock_layout.right_dock_width;

    if let Some(ref bottom_tree) = dock_layout.bottom_dock_state {
        if let Ok(dock_state) = from_str(bottom_tree) {
            bottom_dock_state.dock_state = dock_state;
        }
    }

    bottom_dock_state.height = dock_layout.bottom_dock_height;

    log!(
        LogType::Editor,
        LogLevel::OK,
        LogCategory::UI,
        "Dock State Loaded"
    );
}

fn save_dock_layout_toml(
    editor_state: EditorState,
    right_dock: SideDockState,
    bottom_dock: BottomDockState,
) {
    let save = right_dock
        .dock_state
        .iter_all_tabs()
        .find_map(|(_, tab)| match tab {
            SideTab::EditorSettings { data, .. } => Some(data.dock.store_position_on_close),
            _ => None,
        })
        .unwrap_or(false);

    if !save {
        return;
    }

    let config_path_buf =
        FileAssetReader::get_base_path().join("assets/".to_string() + &editor_state.config_path);
    let dock_layout = get_dock_state_str(right_dock, bottom_dock);

    if let Some(config_path_str) = config_path_buf.to_str() {
        match update_dock_layout_in_config(&dock_layout, config_path_str) {
            Ok(()) => {
                log!(
                    LogType::Editor,
                    LogLevel::OK,
                    LogCategory::System,
                    "Successfully saved dock layout to {}",
                    config_path_str
                );
            }
            Err(e) => {
                log!(
                    LogType::Editor,
                    LogLevel::Error,
                    LogCategory::System,
                    "Failed to save dock layout: {}",
                    e
                );
            }
        }
    } else {
        log!(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::System,
            "Failed to save dock layout to {:?}",
            config_path_buf
        );
    }
}

pub fn update_dock_layout_in_config(
    dock_layout: &DockLayoutStr,
    path: &str,
) -> std::io::Result<()> {
    let mut config: EditorSettingsTabData = load_from_toml_file(path).unwrap_or_default();
    config.dock.layout_str = dock_layout.clone();
    save_to_toml_file(&config, path)
}
