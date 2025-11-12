use super::{AvailableEditableMaterials, SceneAsset, SceneAssetLoader, StringAsset, StringAssetLoader};
use crate::EditableMaterial;
use bevy::{
    app::{App, Plugin, PreStartup},
    asset::{AssetApp, AssetServer, Assets, Handle},
    ecs::system::{Res, ResMut},
    pbr::StandardMaterial,
    prelude::Resource,
};

/// Resource to keep scene asset handles alive for WASM.
/// In WASM, assets need their handles to be kept alive or they'll be unloaded.
/// Add your preloaded scene handles to this resource to keep them in memory.
#[derive(Resource, Default)]
pub struct PreloadedSceneHandles {
    pub handles: Vec<Handle<SceneAsset>>,
}

impl PreloadedSceneHandles {
    /// Add a scene handle to keep it alive
    pub fn add(&mut self, handle: Handle<SceneAsset>) {
        self.handles.push(handle);
    }
    
    /// Preload a scene and keep its handle
    pub fn preload(&mut self, asset_server: &AssetServer, path: impl Into<String>) {
        let handle = asset_server.load(path.into());
        self.handles.push(handle);
    }
}

fn preload_fallback_material(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut available_materials: ResMut<AvailableEditableMaterials>,
    asset_server: Res<AssetServer>,
) {
    // Special None material
    let mut none_material = EditableMaterial::default();
    none_material.set_to_empty();
    none_material.update_name("None".to_string());

    // Fallback editable default white
    let mut white_editable = EditableMaterial::get_new_unnamed_base_color();
    let white_handle: Handle<StandardMaterial> = materials.add(StandardMaterial::default());
    white_editable.set_handle(Some(white_handle));
    white_editable.update_name("default".to_string());
    white_editable.update_path("materials/internal/default.mat".to_string());
    white_editable.material_exists_and_load(
        &mut available_materials,
        &mut materials,
        &asset_server,
        "",
        "",
    );

    if let Some(ref mut materials) = available_materials.materials {
        materials.insert(0, none_material);
        materials.insert(1, white_editable);
    } else {
        available_materials.materials = Some(vec![none_material, white_editable]);
    }
}

pub struct AssetPlugin;
impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Asset Loaders
            //
            .init_asset::<SceneAsset>()
            .init_asset_loader::<SceneAssetLoader>()
            .init_asset::<StringAsset>()
            .init_asset_loader::<StringAssetLoader>()
            //
            // Resources
            //
            .insert_resource(AvailableEditableMaterials::default())
            .insert_resource(PreloadedSceneHandles::default())
            //
            // Schedule system
            //
            .add_systems(PreStartup, preload_fallback_material);
    }
}
