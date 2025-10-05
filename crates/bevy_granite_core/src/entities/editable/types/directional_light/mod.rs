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

use crate::{
    entities::editable::{GraniteType, RequestEntityUpdateFromClass},
    entities::EntitySaveReadyData,
    AvailableEditableMaterials,
};
use crate::{ClassCategory, PromptData};
pub use plugin::*;
pub use update_event::*;

/// Internal event thats called when user edits UI directional light variable
#[derive(Message)]
pub struct UserUpdatedDirectionalLightEvent {
    pub entity: Entity,
    pub data: DirLight,
}

/// Actual serialized class data thats stored inside IdentityData
/// Some actual Bevy Directional light data, some custom flags for easy inserting of additional data  
#[derive(Serialize, Deserialize, Reflect, Debug, Clone, PartialEq)]
pub struct DirLight {
    pub color: (f32, f32, f32),
    pub illuminance: f32,
    pub shadows_enabled: bool,
    // Flag that gets used to insert VolumetricLight
    pub volumetric: bool,
}
impl Default for DirLight {
    fn default() -> Self {
        Self {
            color: (1., 1., 1.),
            illuminance: 32_000.0,
            shadows_enabled: true,
            volumetric: false,
        }
    }
}

impl GraniteType for DirLight {
    fn type_name(&self) -> String {
        "Directional Light".to_string()
    }

    fn type_abv(&self) -> String {
        "Dir. Light".to_string()
    }

    fn category(&self) -> ClassCategory {
        ClassCategory::Light
    }

    fn get_embedded_icon_bytes(&self) -> Option<&'static [u8]> {
        Some(include_bytes!("DirectionalLight.png"))
    }

    fn get_icon_filename(&self) -> Option<&'static str> {
        Some("DirectionalLight.png")
    }

    fn spawn_from_new_identity(
        &mut self,
        commands: &mut Commands,
        transform: Transform,
        _standard_materials: ResMut<Assets<StandardMaterial>>,
        _meshes: ResMut<Assets<Mesh>>,
        _available_materials: ResMut<AvailableEditableMaterials>,
        _asset_server: Res<AssetServer>,
        _maybe_prompt_data: Option<PromptData>,
    ) -> Entity {
        DirLight::spawn_from_new_identity(self, commands, transform)
    }

    fn spawn_from_save_data(
        &self,
        save_data: &EntitySaveReadyData,
        commands: &mut Commands,
        _standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        _meshes: &mut ResMut<Assets<Mesh>>,
        _available_materials: &mut ResMut<AvailableEditableMaterials>,
        _asset_server: &Res<AssetServer>,
    ) -> Entity {
        DirLight::spawn_from_save_data(save_data, commands)
    }

    fn push_to_entity(&self, entity: Entity, request_update: &mut RequestEntityUpdateFromClass) {
        self.push_to_entity(entity, request_update)
    }

    fn edit_via_ui(&mut self, ui: &mut egui::Ui, spacing: (f32, f32, f32)) -> bool {
        self.edit_via_ui(ui, spacing)
    }
}
