#[cfg(not(target_arch = "wasm32"))]
use crate::{LogCategory, LogEntry};

#[cfg(not(target_arch = "wasm32"))]
use std::fs::OpenOptions;

#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

#[cfg(not(target_arch = "wasm32"))]
static LOG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn get_log_path() -> &'static PathBuf {
    LOG_FILE_PATH.get_or_init(|| {
        let path = dirs::config_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .join("bevy_granite_logging")
            .join("app.log");

        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Failed to create log directory: {}", e);
            }
        }

        path
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_to_file(entry: &LogEntry) -> Result<(), std::io::Error> {
    if entry.category == LogCategory::Blank {
        return Ok(()); // Skip blank log entries
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(get_log_path())?;

    let log_line = format!(
        "{}{:?} {:?} {:?} {}\n",
        entry.timestamp, entry.log_type, entry.level, entry.category, entry.message
    );

    file.write_all(log_line.as_bytes())?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn write_to_file(_entry: &crate::LogEntry) -> Result<(), std::io::Error> {
    // No-op on WASM - file system writes not supported
    Ok(())
}
