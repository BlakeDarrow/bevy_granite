// Custom asset type for loading text files as strings through Bevy's AssetServer
// This works on both native and WASM platforms

use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};
use bevy::reflect::TypePath;

#[cfg(target_arch = "wasm32")]
use bevy_granite_logging::{config::{LogCategory, LogLevel, LogType}, log};

/// A simple asset that contains the text contents of a file
#[derive(Asset, TypePath, Debug, Clone)]
pub struct StringAsset {
    pub contents: String,
}

impl StringAsset {
    pub fn new(contents: String) -> Self {
        Self { contents }
    }
}

#[derive(Default)]
pub struct StringAssetLoader;

impl AssetLoader for StringAssetLoader {
    type Asset = StringAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        
        let contents = String::from_utf8(bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        #[cfg(target_arch = "wasm32")]
        {
            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::System,
                "StringAssetLoader: Loading string asset, content length: {}",
                contents.len()
            );
        }
        
        Ok(StringAsset { contents })
    }

    fn extensions(&self) -> &[&str] {
        // NOTE: Do NOT include "scene" - that's handled by SceneAssetLoader
        &["txt", "mat", "toml", "json", "xml", "md", "ron"]
    }
}
