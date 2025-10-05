use std::borrow::Cow;

use crate::{
    entities::editable::{
        GraniteType, RequestEntityUpdateFromClass, RequiredMaterialData, RequiredMaterialDataMut,
    },
    ClassCategory, MaterialData, PromptData,
};
use crate::{entities::EntitySaveReadyData, AvailableEditableMaterials};
use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        entity::Entity,
        message::Message,
        system::{Commands, Res, ResMut},
    },
    mesh::Mesh,
    pbr::StandardMaterial,
    prelude::Reflect,
    transform::components::Transform,
};
use bevy_egui::egui;
use serde::{Deserialize, Serialize};

pub mod creation;
pub mod plugin;
pub mod ui;
pub mod update_event;

pub use plugin::*;
pub use update_event::*;

/// Internal event thats called when user edits UI OBJ variables
#[derive(Message)]
pub struct UserUpdatedOBJEvent {
    pub entity: Entity,
    pub data: OBJ,
    pub reload_mesh: bool,
}

/// Actual serialized class data thats stored inside IdentityData
/// mesh_path is relative disk path to .obj
/// OBJ needs materials, so we pass the required MaterialData which contains, path, current and last materials
#[derive(Serialize, Deserialize, Reflect, Debug, Clone, PartialEq)]
pub struct OBJ {
    pub mesh_path: Cow<'static, str>,
    pub material: MaterialData,
    #[serde(skip)]
    pub reload_requested: bool,
}
impl Default for OBJ {
    fn default() -> Self {
        Self {
            mesh_path: "".into(),
            material: MaterialData::new("".to_string()),
            reload_requested: false,
        }
    }
}

/// GraniteType contains all the needed function to define out custom editor editable type
impl GraniteType for OBJ {
    fn category(&self) -> ClassCategory {
        ClassCategory::Mesh
    }

    fn type_name(&self) -> String {
        "OBJ".to_string()
    }

    fn type_abv(&self) -> String {
        "OBJ".to_string()
    }

    fn needs_prompt(&self) -> bool {
        true
    }

    fn get_prompt_config(&self) -> (String, Vec<&'static str>) {
        ("models".to_string(), vec!["obj"])
    }

    fn spawn_from_new_identity(
        &mut self,
        commands: &mut Commands,
        transform: Transform,
        mut standard_materials: ResMut<Assets<StandardMaterial>>,
        _meshes: ResMut<Assets<Mesh>>,
        mut available_materials: ResMut<AvailableEditableMaterials>,
        asset_server: Res<AssetServer>,
        maybe_prompt_data: Option<PromptData>,
    ) -> Entity {
        self.spawn_from_new_identity(
            commands,
            transform,
            &mut standard_materials,
            &mut available_materials,
            &asset_server,
            maybe_prompt_data,
        )
    }

    fn spawn_from_save_data(
        &self,
        save_data: &EntitySaveReadyData,
        commands: &mut Commands,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        _meshes: &mut ResMut<Assets<Mesh>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
    ) -> Entity {
        OBJ::spawn_from_save_data(
            save_data,
            commands,
            standard_materials,
            available_materials,
            asset_server,
        )
    }

    fn push_to_entity(&self, entity: Entity, request_update: &mut RequestEntityUpdateFromClass) {
        self.push_to_entity(entity, request_update)
    }

    fn edit_via_ui(&mut self, ui: &mut egui::Ui, spacing: (f32, f32, f32)) -> bool {
        self.edit_via_ui(ui, spacing)
    }

    fn get_material_data(&self) -> Option<RequiredMaterialData> {
        Some(self.material.as_ref())
    }

    fn get_mut_material_data(&mut self) -> Option<RequiredMaterialDataMut> {
        Some(self.material.as_mut())
    }
}
