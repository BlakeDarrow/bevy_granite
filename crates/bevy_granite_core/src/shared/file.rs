#[cfg(all(not(target_arch = "wasm32"), not(feature = "bundler")))]
use bevy::asset::io::file::FileAssetReader;

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

// With bundler, just return relative paths as-is since assets are embedded
#[cfg(all(not(target_arch = "wasm32"), feature = "bundler"))]
pub fn rel_asset_to_absolute(rel_string: &str) -> Cow<'static, str> {
    let normalized_rel = rel_string.replace('\\', "/");
    normalized_rel.into()
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "bundler")))]
pub fn rel_asset_to_absolute(rel_string: &str) -> Cow<'static, str> {
    let normalized_rel = rel_string.replace('\\', "/");
    
    let abs_path: PathBuf = if !Path::new(&normalized_rel).is_absolute() {
        FileAssetReader::get_base_path()
            .join("assets")
            .join(&normalized_rel)
    } else {
        PathBuf::from(&normalized_rel)
    };

    abs_path.to_string_lossy().replace('\\', "/").into()
}

#[cfg(target_arch = "wasm32")]
pub fn rel_asset_to_absolute(rel_string: &str) -> Cow<'static, str> {
    let normalized_rel = rel_string.replace('\\', "/");
    
    let abs_path: PathBuf = if !Path::new(&normalized_rel).is_absolute() {
        PathBuf::from("assets").join(&normalized_rel)
    } else {
        PathBuf::from(&normalized_rel)
    };

    abs_path.to_string_lossy().replace('\\', "/").into()
}

// With bundler, just normalize and return the path as-is
#[cfg(all(not(target_arch = "wasm32"), feature = "bundler"))]
pub fn absolute_asset_to_rel(abs_string: String) -> Cow<'static, str> {
    let normalized = abs_string.replace('\\', "/");
    
    // Strip "assets/" prefix if present
    if let Some(stripped) = normalized.strip_prefix("assets/") {
        stripped.to_string().into()
    } else {
        normalized.into()
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "bundler")))]
pub fn absolute_asset_to_rel(abs_string: String) -> Cow<'static, str> {
    let abs_path = Path::new(&abs_string).canonicalize().unwrap_or_else(|_| PathBuf::from(&abs_string));

    let base_assets_path = FileAssetReader::get_base_path()
        .join("assets")
        .canonicalize()
        .unwrap_or_else(|_| FileAssetReader::get_base_path().join("assets"));

    if abs_path.starts_with(&base_assets_path) {
        abs_path
            .strip_prefix(&base_assets_path)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
            .into()
    } else {
        abs_path.to_string_lossy().replace('\\', "/").into()
    }
}

#[cfg(target_arch = "wasm32")]
pub fn absolute_asset_to_rel(abs_string: String) -> Cow<'static, str> {
    let abs_path = Path::new(&abs_string);
    let base_assets_path = PathBuf::from("assets");

    if let Ok(stripped) = abs_path.strip_prefix(&base_assets_path) {
        stripped.to_string_lossy().replace('\\', "/").into()
    } else {
        abs_path.to_string_lossy().replace('\\', "/").into()
    }
}
