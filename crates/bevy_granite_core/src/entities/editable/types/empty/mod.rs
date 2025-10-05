use crate::{
    entities::{
        editable::{GraniteType, RequestEntityUpdateFromClass},
        EntitySaveReadyData,
    },
    AvailableEditableMaterials, ClassCategory, PromptData,
};
use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        entity::Entity,
        message::Message,
        system::{Commands, Res, ResMut},
    },
    mesh::Mesh,
    pbr::StandardMaterial,
    reflect::Reflect,
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

// We have no event here as there isnt data that can be edited via UI
/// Internal event thats called when user edits UI directional light variable
#[derive(Message)]
pub struct UserUpdatedEmptyEvent {
    pub entity: Entity,
    pub data: Empty,
}

/// Actual serialized class data thats stored inside IdentityData
/// In this case its a unit struct as Empty - Spatial Bundle needs nothing
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub struct Empty {
    // Empty
}

impl GraniteType for Empty {
    fn type_name(&self) -> String {
        "Empty".to_string()
    }

    fn type_abv(&self) -> String {
        "Empty".to_string()
    }

    fn category(&self) -> ClassCategory {
        ClassCategory::Empty
    }

    fn get_embedded_icon_bytes(&self) -> Option<&'static [u8]> {
        Some(include_bytes!("Empty.png"))
    }

    fn get_icon_filename(&self) -> Option<&'static str> {
        Some("Empty.png")
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
        Empty::spawn_from_new_identity(self, commands, transform)
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
        Empty::spawn_from_save_data(save_data, commands)
    }

    fn push_to_entity(&self, entity: Entity, request_update: &mut RequestEntityUpdateFromClass) {
        self.push_to_entity(entity, request_update)
    }

    fn edit_via_ui(&mut self, ui: &mut egui::Ui, spacing: (f32, f32, f32)) -> bool {
        self.edit_via_ui(ui, spacing)
    }
}
