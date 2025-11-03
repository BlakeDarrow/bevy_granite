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

/// Internal event thats called when user edits UI spot light variable
#[derive(Message)]
pub struct UserUpdatedSpotLightEvent {
    pub entity: Entity,
    pub data: SpotLightData,
}

/// Actual serialized class data thats stored inside IdentityData
/// In this case, all the vars are direct references for SpotLight Bevy data
#[derive(Serialize, Deserialize, Reflect, Debug, Clone, PartialEq)]
pub struct SpotLightData {
    pub color: (f32, f32, f32),
    pub intensity: f32,
    pub range: f32,
    pub radius: f32,
    pub shadows_enabled: bool,
    pub shadow_depth_bias: f32,
    pub shadow_normal_bias: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

impl Default for SpotLightData {
    fn default() -> Self {
        Self {
            color: (1., 1., 1.),
            intensity: 750_000.0,
            range: 20.0,
            radius: 0.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 1.8,
            inner_angle: 0.0,
            outer_angle: std::f32::consts::PI / 4.0, // 45 degrees
        }
    }
}

impl GraniteType for SpotLightData {
    fn type_name(&self) -> String {
        "Spot Light".to_string()
    }

    fn type_abv(&self) -> String {
        "S.Light".to_string()
    }

    fn category(&self) -> ClassCategory {
        ClassCategory::Light
    }

    fn get_embedded_icon_bytes(&self) -> Option<&'static [u8]> {
        Some(include_bytes!("SpotLight.png"))
    }

    fn get_icon_filename(&self) -> Option<&'static str> {
        Some("SpotLight.png")
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
        SpotLightData::spawn_from_new_identity(self, commands, transform)
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
        SpotLightData::spawn_from_save_data(save_data, commands)
    }

    fn push_to_entity(&self, entity: Entity, request_update: &mut RequestEntityUpdateFromClass) {
        SpotLightData::push_to_entity(self, entity, request_update);
    }

    fn edit_via_ui(&mut self, ui: &mut egui::Ui, spacing: (f32, f32, f32)) -> bool {
        SpotLightData::edit_via_ui(self, ui, spacing)
    }
}
