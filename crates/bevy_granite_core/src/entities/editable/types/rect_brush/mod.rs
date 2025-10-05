use crate::{
    assets::EditableMaterial,
    entities::{
        editable::{
            GraniteType, RequestEntityUpdateFromClass, RequiredMaterialData,
            RequiredMaterialDataMut,
        },
        EntitySaveReadyData, PromptData,
    },
    AvailableEditableMaterials, ClassCategory, MaterialData,
};
use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        entity::Entity,
        message::Message,
        system::{Commands, Res, ResMut},
    },
    math::{Vec2, Vec3},
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

/// Internal event thats called when user edits UI RectBrush variables
#[derive(Message)]
pub struct UserUpdatedRectBrushEvent {
    pub entity: Entity,
    pub data: RectBrush,
}

/// Actual serialized class data thats stored inside IdentityData
/// Size is x,y,z that actually edits the verts of this brush, not just a scale
/// UV scale is mapped directly to the verts, so this is separate then Material editing UVs
/// Rectangle Brushes contain materials on their surface so we pass the path, last, and current material under MaterialData
#[derive(Serialize, Deserialize, Reflect, Debug, Clone, PartialEq)]
pub struct RectBrush {
    pub size: Vec3,
    pub uv_scale: Vec2,
    pub material: MaterialData,
}

impl RectBrush {
    // Needed static access to the type name so we define them static-ly here this time
    pub fn type_name_static() -> String {
        "Rectangle Brush".to_string()
    }

    pub fn type_abv_static() -> String {
        "Brush".to_string()
    }

    pub fn internal_material_path() -> String {
        "materials/internal/rect_brush.mat".to_string()
    }
}

impl Default for RectBrush {
    fn default() -> Self {
        let (path, name) = (
            Self::internal_material_path(),
            "Rectangle Brush".to_string(),
        );

        // Create a material with the internal defaults for rectangle brush
        let mut brush_material = EditableMaterial::get_new_unnamed_base_color();
        brush_material.update_name(name.clone());
        brush_material.update_path(path.clone());

        Self {
            size: Vec3::ONE,
            uv_scale: Vec2::ONE,
            material: MaterialData {
                path: path.clone(),
                current: brush_material.clone(),
                last: brush_material.clone(),
            },
        }
    }
}

impl GraniteType for RectBrush {
    fn category(&self) -> ClassCategory {
        ClassCategory::Mesh
    }

    fn type_name(&self) -> String {
        RectBrush::type_name_static()
    }

    fn type_abv(&self) -> String {
        RectBrush::type_abv_static()
    }

    fn spawn_from_new_identity(
        &mut self,
        commands: &mut Commands,
        transform: Transform,
        mut standard_materials: ResMut<Assets<StandardMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut available_materials: ResMut<AvailableEditableMaterials>,
        asset_server: Res<AssetServer>,
        _maybe_prompt_data: Option<PromptData>,
    ) -> Entity {
        self.spawn_from_new_identity(
            commands,
            transform,
            &mut standard_materials,
            &mut available_materials,
            &asset_server,
            &mut meshes,
        )
    }

    fn spawn_from_save_data(
        &self,
        save_data: &EntitySaveReadyData,
        commands: &mut Commands,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
    ) -> Entity {
        RectBrush::spawn_from_save_data(
            save_data,
            commands,
            standard_materials,
            available_materials,
            asset_server,
            meshes,
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

    fn needs_unique_handle(&self) -> bool {
        true
    }
}
