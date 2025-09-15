use bevy::asset::io::file::FileAssetReader;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

pub fn rel_asset_to_absolute(rel_string: &str) -> Cow<'static, str> {
    let abs_path: PathBuf = if !Path::new(rel_string).is_absolute() {
        FileAssetReader::get_base_path()
            .join("assets")
            .join(rel_string)
    } else {
        PathBuf::from(rel_string)
    };

    abs_path.to_string_lossy().replace('\\', "/").into()
}

pub fn absolute_asset_to_rel(abs_string: String) -> Cow<'static, str> {
    let abs_path = Path::new(&abs_string);

    if abs_path.is_absolute() {
        let base_assets_path = FileAssetReader::get_base_path().join("assets");

        if let Ok(rel_path) = abs_path.strip_prefix(&base_assets_path) {
            rel_path.to_string_lossy().replace('\\', "/").into()
        } else {
            abs_string.into()
        }
    } else {
        abs_string.into()
    }
}
