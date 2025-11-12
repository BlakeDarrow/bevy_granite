use serde::{de::DeserializeOwned, Serialize};
use std::io::{self, Result};

#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
pub fn load_from_toml_file<T: DeserializeOwned>(path: &str) -> Result<T> {
    let content = fs::read_to_string(path)?;
    let data: T =
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(data)
}

#[cfg(target_arch = "wasm32")]
pub fn load_from_toml_file<T: DeserializeOwned>(_path: &str) -> Result<T> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "File operations not supported on WASM",
    ))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_to_toml_file<T: Serialize>(value: &T, path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let toml_str =
        toml::to_string_pretty(value).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(path, toml_str)
}

#[cfg(target_arch = "wasm32")]
pub fn save_to_toml_file<T: Serialize>(_value: &T, _path: &str) -> Result<()> {
    // No-op on WASM - file system writes not supported
    Ok(())
}
