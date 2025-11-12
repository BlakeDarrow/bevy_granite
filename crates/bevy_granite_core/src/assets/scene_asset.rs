// Custom asset type for loading .scene files through Bevy's AssetServer
// This works on both native and WASM platforms

use bevy::asset::{Asset, AssetLoader, AsyncReadExt, LoadContext};
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::{IdentityData, TransformData};
use crate::shared::version::Version;

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct SceneAsset {
    pub metadata: SceneMetadata,
    pub entities: Vec<EntityData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMetadata {
    pub format_version: Version,
    pub entity_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub identity: IdentityData,
    pub transform: TransformData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<HashMap<String, String>>,
}

#[derive(Default)]
pub struct SceneAssetLoader;

impl AssetLoader for SceneAssetLoader {
    type Asset = SceneAsset;
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
        let content = String::from_utf8(bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        ron::de::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn extensions(&self) -> &[&str] {
        &["scene"]
    }
}
