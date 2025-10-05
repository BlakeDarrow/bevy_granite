use crate::{
    entities::editable::{GraniteType, RequestEntityUpdateFromClass},
    entities::EntitySaveReadyData,
    AvailableEditableMaterials,
};
use crate::{ClassCategory, PromptData};
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

/// Internal event thats called when user edits UI point light variable
#[derive(Message)]
pub struct UserUpdatedPointLightEvent {
    pub entity: Entity,
    pub data: PointLightData,
}

/// Actual serialized class data thats stored inside IdentityData
/// In this case, all the vars are direct references for PointLight Bevy data
#[derive(Serialize, Deserialize, Reflect, Debug, Clone, PartialEq)]
pub struct PointLightData {
    pub intensity: f32,
    pub range: f32,
    pub shadows_enabled: bool,
    pub color: (f32, f32, f32),
}
impl Default for PointLightData {
    fn default() -> Self {
        Self {
            intensity: 400_000.0,
            range: 10.0,
            shadows_enabled: true,
            color: (1., 1., 1.),
        }
    }
}

impl GraniteType for PointLightData {
    fn type_name(&self) -> String {
        "Point Light".to_string()
    }

    fn type_abv(&self) -> String {
        "P.Light".to_string()
    }

    fn category(&self) -> ClassCategory {
        ClassCategory::Light
    }

    fn get_embedded_icon_bytes(&self) -> Option<&'static [u8]> {
        Some(include_bytes!("PointLight.png"))
    }

    fn get_icon_filename(&self) -> Option<&'static str> {
        Some("PointLight.png")
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
        PointLightData::spawn_from_new_identity(self, commands, transform)
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
        PointLightData::spawn_from_save_data(save_data, commands)
    }

    fn push_to_entity(&self, entity: Entity, request_update: &mut RequestEntityUpdateFromClass) {
        self.push_to_entity(entity, request_update)
    }

    fn edit_via_ui(&mut self, ui: &mut egui::Ui, spacing: (f32, f32, f32)) -> bool {
        self.edit_via_ui(ui, spacing)
    }
}
