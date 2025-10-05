use crate::{
    entities::EntitySaveReadyData, AvailableEditableMaterials, PromptData, RequiredMaterialData,
    RequiredMaterialDataMut,
};
use bevy::{
    asset::{AssetId, AssetServer, Assets, Handle},
    ecs::{
        entity::Entity,
        system::{Commands, Res, ResMut},
    },
    ecs::{message::MessageWriter, system::SystemParam},
    mesh::Mesh,
    pbr::StandardMaterial,
    prelude::{Image, Reflect},
    transform::components::Transform,
};
use bevy_egui::egui;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

// Modules
pub mod category;
pub mod definition;
pub mod types;

// Re-exports
pub use category::ClassCategory;
pub use definition::GraniteType;
pub use types::*;

// ---------------------------------------------------------------------------------------
// Callbacks to update the real entity from UI changes -> .push_to_entity()

#[derive(SystemParam)]
pub struct RequestEntityUpdateFromClass<'w> {
    pub camera_3d: MessageWriter<'w, UserUpdatedCamera3DEvent>,
    pub directional_light: MessageWriter<'w, UserUpdatedDirectionalLightEvent>,
    pub point_light: MessageWriter<'w, UserUpdatedPointLightEvent>,
    pub rectangle_brush: MessageWriter<'w, UserUpdatedRectBrushEvent>,
    pub obj: MessageWriter<'w, UserUpdatedOBJEvent>,
    pub empty: MessageWriter<'w, UserUpdatedEmptyEvent>,
}

// ---------------------------------------------------------------------------------------

// GraniteTypes represents all Granite entity "types" that are supported
// This should be relatively trivial to expand with new types
// If you add a new type ensure to add it under the enum and its all function
/// We use enum_dispatch for static polymorphism - i.e. all variants of our enum need same functions available to themselves, and exposed up a level - this saves us a tremendous amount of match arms in this enum
#[enum_dispatch(GraniteType)]
#[derive(Serialize, Reflect, Deserialize, PartialEq, Clone, Debug)]
pub enum GraniteTypes {
    OBJ(OBJ),
    Empty(Empty),
    PointLightData(PointLightData),
    DirLight(DirLight),
    Camera3D(Camera3D),
    RectBrush(RectBrush),
    Unknown(Unknown), // Holds no real data
}
impl GraniteTypes {
    // If you add a new custom type - add it here as well so its concretely known!!
    // Used to get all available variants, not data
    pub fn all() -> Vec<GraniteTypes> {
        vec![
            GraniteTypes::OBJ(Default::default()),
            GraniteTypes::Empty(Default::default()),
            GraniteTypes::PointLightData(Default::default()),
            GraniteTypes::DirLight(Default::default()),
            GraniteTypes::Camera3D(Default::default()),
            GraniteTypes::RectBrush(Default::default()),
            GraniteTypes::Unknown(Default::default()),
        ]
    }

    // Check if we are a known type - helpful for UI and what not
    pub fn is_known(&self) -> bool {
        !matches!(self, GraniteTypes::Unknown(_))
    }

    // Return vector of all GraniteTypes variant categories. Used for UI
    pub fn all_by_category(category: ClassCategory) -> Vec<GraniteTypes> {
        Self::all()
            .into_iter()
            .filter(|class_type| class_type.category() == category)
            .collect()
    }
}

impl Default for GraniteTypes {
    fn default() -> Self {
        Self::Unknown(Default::default())
    }
}

// ---------------------------------------------------------------------------------------
