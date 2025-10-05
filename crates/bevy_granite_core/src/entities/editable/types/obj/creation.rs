use super::OBJ;
use crate::{
    absolute_asset_to_rel, entities::EntitySaveReadyData, shared::rel_asset_to_absolute,
    AvailableEditableMaterials, GraniteEditorSerdeEntity, GraniteTypes, HasRuntimeData,
    IdentityData, MaterialNameSource, NeedsTangents, PromptData, PromptImportSettings,
};
use bevy::{
    asset::{AssetPath, AssetServer, Assets, Handle},
    ecs::{
        bundle::Bundle,
        entity::Entity,
        system::{Commands, Res, ResMut},
    },
    mesh::{Mesh, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::Name,
    transform::components::Transform,
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
use uuid::Uuid;

impl OBJ {
    /// Extract needed info to spawn this entity via save data
    pub fn spawn_from_save_data(
        save_data: &EntitySaveReadyData,
        commands: &mut Commands,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
    ) -> Entity {
        let identity = &save_data.identity;
        let save_transform = &save_data.transform;

        let prompt_settings = PromptImportSettings {
            create_mat_on_import: true,
            material_name_source: MaterialNameSource::SaveData, // to ensure we get the proper identities class's found material path
        };

        Self::spawn_from_identity(
            commands,
            identity,
            save_transform.to_bevy(),
            standard_materials,
            available_materials,
            asset_server,
            prompt_settings.create_mat_on_import, // Should we load the material?
            prompt_settings.material_name_source, // where should material name come from
        )
    }

    /// Take the name and class from identity to spawn
    pub fn spawn_from_identity(
        commands: &mut Commands,
        identity: &IdentityData,
        transform: Transform,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
        create_material: bool,
        create_material_from: MaterialNameSource,
    ) -> Entity {
        let mut class_data = Self::extract_class(identity);

        class_data.spawn(
            identity,
            commands,
            transform,
            standard_materials,
            available_materials,
            asset_server,
            create_material,      // Should we load the material?
            create_material_from, // Where should the new material name come from
        )
    }

    /// Generally to be used from UI popups - spawns with new identity
    /// In this case the maybe_prompt_data is either prompted file location or directly passed
    pub fn spawn_from_new_identity(
        &mut self,
        commands: &mut Commands,
        transform: Transform,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
        maybe_prompt_data: Option<PromptData>,
    ) -> Entity {
        // Prompt info has the desired disk path to load mesh from, and prompt import settings
        let prompt_info = maybe_prompt_data.unwrap_or_default();
        let file_path = prompt_info.file.unwrap_or(self.mesh_path.to_string());

        // Ensure we have rel and abs
        let rel_path = absolute_asset_to_rel(file_path);
        let abs_path = rel_asset_to_absolute(&rel_path);

        // Extract name from OBJ file
        let entity_name = match Self::extract_first_object_name(abs_path.as_ref()) {
            Ok(name) => name,
            Err(_) => "Imported Object".to_string(),
        };

        // Update internal state
        self.mesh_path = rel_path;

        let identity = IdentityData {
            name: entity_name,
            uuid: Uuid::new_v4(),
            class: GraniteTypes::OBJ(self.clone()),
        };

        self.spawn(
            &identity,
            commands,
            transform,
            standard_materials,
            available_materials,
            asset_server,
            prompt_info.import_settings.create_mat_on_import, // Should we create materials for imported objs
            prompt_info.import_settings.material_name_source, // If we are creating materials, where does the name come from?
        )
    }

    /// Private core logic
    fn spawn(
        &mut self,
        identity: &IdentityData,
        commands: &mut Commands,
        transform: Transform,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
        create_material: bool,
        create_material_from: MaterialNameSource,
    ) -> Entity {
        //log!(
        //    LogType::Game,
        //    LogLevel::Info,
        //    LogCategory::Asset,
        //    "User wants obj from: {}",
        //    self.mesh_path,
        //);

        // Load and configure the material
        self.load_and_configure_material(
            identity,
            available_materials,
            standard_materials,
            asset_server,
            create_material,
            create_material_from,
        );

        let path = self.mesh_path.to_string();

        let mesh_handle: Handle<Mesh> = asset_server.load(path);

        commands
            .spawn(Self::get_bundle(
                self.clone(),
                identity.clone(),
                transform,
                mesh_handle,
            ))
            .id()
    }

    fn get_bundle(
        obj: OBJ,
        identity: IdentityData,
        transform: Transform,
        mesh_handle: Handle<Mesh>,
    ) -> impl Bundle {
        (
            transform,
            Mesh3d(mesh_handle),
            MeshMaterial3d(
                obj.material
                    .current
                    .handle
                    .clone()
                    .expect("This obj should always have a handle"),
            ),
            Name::new(identity.name.clone()),
            HasRuntimeData,
            GraniteEditorSerdeEntity,
            NeedsTangents,
            IdentityData {
                name: identity.name.clone(),
                uuid: identity.uuid.clone(),
                class: GraniteTypes::OBJ(obj.clone()), // Use the updated obj, not the old identity.class
            },
        )
    }

    fn extract_first_object_name<P: AsRef<Path>>(obj_path: P) -> Result<String, std::io::Error> {
        let file = File::open(obj_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().take(10) {
            let line = line?;
            if let Some(name) = line.trim().strip_prefix("o ") {
                let name = name.trim();
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No object name found",
        ))
    }

    fn extract_first_usemtl_name<P: AsRef<Path>>(obj_path: P) -> Result<String, std::io::Error> {
        let file = File::open(obj_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(name) = line.trim().strip_prefix("usemtl ") {
                let name = name.trim();
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No object name found",
        ))
    }

    fn extract_class(identity: &IdentityData) -> OBJ {
        match &identity.class {
            GraniteTypes::OBJ(obj_data) => obj_data.clone(),
            _ => panic!("Expected OBJ class data, got different type from save data"),
        }
    }

    /// Load and configure material with proper metadata
    /// Fallback is entity name to create new OBJ material
    fn load_and_configure_material(
        &mut self,
        identity: &IdentityData,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        create_material: bool,
        create_material_from: MaterialNameSource,
    ) {
        let mut material_path: String = String::new();
        let engine_fallback = String::from("materials/internal/default.mat");
        if create_material {
            material_path = {
                let abs_path = rel_asset_to_absolute(&self.mesh_path);
                match create_material_from {
                    MaterialNameSource::SaveData => {
                        // Save data should contain the material path, if it doesnt, dont return a path
                        if !self.material.path.is_empty() && self.material.path != "None" {
                            self.material.path.clone()
                        } else {
                            log!(
                                LogType::Game,
                                LogLevel::Warning,
                                LogCategory::Asset,
                                "Save data did not contain a material path for obj at: {}",
                                abs_path
                            );
                            engine_fallback
                        }
                    }
                    MaterialNameSource::FileName => {
                        format!("materials/{}.mat", identity.name.clone().to_lowercase())
                        // Simple use the file name as material name
                    }
                    MaterialNameSource::DefaultMaterial => engine_fallback,
                    MaterialNameSource::FileContents => {
                        match OBJ::extract_first_usemtl_name(abs_path.as_ref()) {
                            Ok(material_path) => {
                                format!("materials/{}.mat", material_path.to_lowercase())
                                // Extracted a usemtl name from the specified obj file
                            }
                            Err(_) => {
                                // if fallback, we will use the engine default
                                engine_fallback
                            }
                        }
                    }
                }
            };
        };

        //log!(
        //    LogType::Game,
        //    LogLevel::Info,
        //    LogCategory::Asset,
        //    "Create/load material: {}, path: {:?}",
        //    create_material,
        //    material_path
        //);

        // Set the path on the current material before loading
        self.material.current.path = material_path.clone();

        // Ensure our self's material is existing and is loaded. If it doesnt exist, we try the fallback
        self.material.current.material_exists_and_load(
            available_materials,
            standard_materials,
            asset_server,
            // Fallback name and path
            &identity.name,
            &material_path,
        );

        // Fix the material metadata after loading (since loaded materials have "None" path)
        self.material.current.path = material_path.clone();
        self.material.current.friendly_name = identity.name.clone();

        // Always set last = current after material loading
        self.material.last = self.material.current.clone();

        // Only update path if we used fallback (new entity), not when loading from save
        if self.material.path.is_empty() {
            self.material.path = material_path.clone();
        }
    }
}
