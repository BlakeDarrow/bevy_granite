use crate::absolute_asset_to_rel;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use native_dialog::FileDialog;

pub fn asset_file_browser(path: String, filter: Vec<&str>) -> Option<String> {
    let current_dir = std::env::current_dir().unwrap();
    let assets_dir = current_dir.join("assets");
    let location = assets_dir.join(&path);

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::System,
        "asset_file_browser called with path: '{}'",
        path
    );

    // Create the directory if it doesn't exist
    if !location.exists() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "Creating directory: {}",
            location.display()
        );

        if let Err(e) = std::fs::create_dir_all(&location) {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::System,
                "Failed to create directory {}: {}",
                location.display(),
                e
            );
            return None;
        }
    }

    if let Some(selected_path) = FileDialog::new()
        .set_location(&location)
        .add_filter("Files", &filter)
        .show_open_single_file()
        .unwrap()
    {
        if selected_path.starts_with(&assets_dir) {
            Some(selected_path.to_string_lossy().to_string())
        } else {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::System,
                "Cannot select an asset outside of the assets folder!"
            );
            None
        }
    } else {
        None
    }
}

pub fn asset_file_browser_multiple(path: String, filter: Vec<&str>) -> Option<Vec<String>> {
    let current_dir = std::env::current_dir().unwrap();
    let assets_dir = current_dir.join("assets");
    let location = assets_dir.join(&path);

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::System,
        "asset_file_browser_multiple called with path: '{}'",
        path
    );

    // Create the directory if it doesn't exist
    if !location.exists() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "Creating directory: {}",
            location.display()
        );

        if let Err(e) = std::fs::create_dir_all(&location) {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::System,
                "Failed to create directory {}: {}",
                location.display(),
                e
            );
            return None;
        }
    }

    let selected_paths = FileDialog::new()
        .set_location(&location)
        .add_filter("Files", &filter)
        .show_open_multiple_file()
        .unwrap();

    if selected_paths.is_empty() {
        return None;
    }

    let mut valid_paths = Vec::new();

    for path in selected_paths {
        if path.starts_with(&assets_dir) {
            valid_paths.push(absolute_asset_to_rel(path.to_string_lossy().to_string()).to_string());
        } else {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::System,
                "Skipping asset outside of assets folder: {}",
                path.display()
            );
        }
    }

    if valid_paths.is_empty() {
        None
    } else {
        Some(valid_paths)
    }
}
