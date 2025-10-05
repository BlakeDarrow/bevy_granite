use crate::entities::editable::{GraniteType, RequestEntityUpdateFromClass};
use crate::{entities::EntitySaveReadyData, AvailableEditableMaterials};
use bevy::ecs::message::Message;
use bevy::{
    asset::{AssetServer, Assets},
    color::Color,
    ecs::{
        entity::Entity,
        system::{Commands, Res, ResMut},
    },
    mesh::Mesh,
    pbr::StandardMaterial,
    prelude::Reflect,
    transform::components::Transform,
};
use bevy_egui::egui;

use crate::{ClassCategory, PromptData};
use serde::{Deserialize, Serialize};

pub mod creation;
pub mod plugin;
pub mod ui;
pub mod update_event;

pub use plugin::*;
pub use update_event::*;

/// Internal event thats called when user edits UI camera variable
#[derive(Message)]
pub struct UserUpdatedCamera3DEvent {
    pub entity: Entity,
    pub data: Camera3D,
}

/// Actual serialized class data thats stored inside IdentityData
/// is_active is Bevy Camera3D data
/// has_volumetric_fog and counterpart settings are custom to inject volumetrics easier
#[derive(Serialize, Deserialize, Reflect, Debug, Clone, PartialEq)]
pub struct Camera3D {
    pub is_active: bool,
    pub has_volumetric_fog: bool, // if true, our next update even will insert volumetric fog settings

    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumetric_fog_settings: Option<VolumetricFog>,
}
impl Default for Camera3D {
    fn default() -> Self {
        Self {
            is_active: true,
            has_volumetric_fog: false,
            volumetric_fog_settings: None,
        }
    }
}

/// Wrapper for bevy volumetric fog thats serializable and optional
/// Will need to keep in parity if Bevy changes how it stores these settings
#[derive(Serialize, Deserialize, Reflect, Debug, Clone, PartialEq)]
pub struct VolumetricFog {
    pub fog_color: Color,
    pub ambient_color: Color,
    pub ambient_intensity: f32,
    pub step_count: u32,
    pub max_depth: f32,
    pub absorption: f32,
    pub scattering: f32,
    pub density: f32,
    pub scattering_asymmetry: f32,
    pub light_tint: Color,
    pub light_intensity: f32,
}
impl Default for VolumetricFog {
    fn default() -> Self {
        Self {
            fog_color: Color::WHITE,
            ambient_color: Color::WHITE,
            ambient_intensity: 0.1,
            step_count: 64,
            max_depth: 25.0,
            absorption: 0.3,
            scattering: 0.3,
            density: 0.1,
            scattering_asymmetry: 0.8,
            light_tint: Color::WHITE,
            light_intensity: 0.1,
        }
    }
}

impl GraniteType for Camera3D {
    fn type_name(&self) -> String {
        "Camera 3D".to_string()
    }

    fn type_abv(&self) -> String {
        "3D Cam".to_string()
    }

    fn category(&self) -> ClassCategory {
        ClassCategory::Gameplay
    }

    fn get_embedded_icon_bytes(&self) -> Option<&'static [u8]> {
        Some(include_bytes!("Camera.png"))
    }

    fn get_icon_filename(&self) -> Option<&'static str> {
        Some("Camera.png")
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
        Camera3D::spawn_from_new_identity(self, commands, transform)
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
        Camera3D::spawn_from_save_data(save_data, commands)
    }

    fn push_to_entity(&self, entity: Entity, request_update: &mut RequestEntityUpdateFromClass) {
        self.push_to_entity(entity, request_update)
    }

    fn edit_via_ui(&mut self, ui: &mut egui::Ui, spacing: (f32, f32, f32)) -> bool {
        self.edit_via_ui(ui, spacing)
    }
}
