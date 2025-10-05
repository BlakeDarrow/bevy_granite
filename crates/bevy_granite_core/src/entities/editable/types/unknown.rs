use crate::{
    entities::{EntitySaveReadyData, RequestEntityUpdateFromClass},
    AvailableEditableMaterials, ClassCategory, GraniteType, PromptData,
};
use bevy::{
    asset::{AssetServer, Assets},
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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Reflect, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct Unknown {
    _phantom: u8,
}

impl GraniteType for Unknown {
    fn category(&self) -> ClassCategory {
        ClassCategory::Unknown
    }

    fn type_name(&self) -> String {
        "Unknown".to_string()
    }

    fn type_abv(&self) -> String {
        "Unknown".to_string()
    }

    fn spawn_from_new_identity(
        &mut self,
        _commands: &mut Commands,
        _transform: Transform,
        _standard_materials: ResMut<Assets<StandardMaterial>>,
        _meshes: ResMut<Assets<Mesh>>,
        _available_materials: ResMut<AvailableEditableMaterials>,
        _asset_server: Res<AssetServer>,
        _maybe_prompt_data: Option<PromptData>,
    ) -> Entity {
        Entity::PLACEHOLDER
    }

    fn spawn_from_save_data(
        &self,
        _save_data: &EntitySaveReadyData,
        _commands: &mut Commands,
        _standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        _meshes: &mut ResMut<Assets<Mesh>>,
        _available_materials: &mut ResMut<AvailableEditableMaterials>,
        _asset_server: &Res<AssetServer>,
    ) -> Entity {
        Entity::PLACEHOLDER
    }

    fn push_to_entity(&self, _entity: Entity, _request_update: &mut RequestEntityUpdateFromClass) {
        // Empty
    }

    fn edit_via_ui(&mut self, _ui: &mut egui::Ui, _spacing: (f32, f32, f32)) -> bool {
        false
    }
}
