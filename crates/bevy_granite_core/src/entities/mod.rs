use std::borrow::Cow;

use bevy::{
    ecs::{component::Component, resource::Resource},
    prelude::{
        Quat, ReflectComponent, ReflectDefault, ReflectDeserialize, ReflectFromReflect,
        ReflectSerialize, Vec3,
    },
    reflect::Reflect,
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod component_editor;
pub mod deserialize;
pub mod editable;
pub mod generate_tangents;
pub mod lifecycle;
pub mod plugin;
pub mod serialize;
pub use editable::*;

/// Main camera
#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Component, Default, PartialEq)]
#[reflect(Component, Serialize, Deserialize, Default, FromReflect)]
pub struct MainCamera;

#[derive(Reflect, Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[reflect(Serialize, Deserialize, FromReflect)]
pub enum SaveSettings {
    PreserveDiskFull,
    PreserveDiskTransform,
    #[default]
    Runtime,
}

/// Tracks the source/origin of an entity
/// String is relative path from /assets
#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Component, Default, PartialEq)]
#[reflect(Component, Serialize, Deserialize, Default, FromReflect)]
pub struct SpawnSource(Cow<'static, str>, SaveSettings);
impl SpawnSource {
    pub fn new(path: impl Into<Cow<'static, str>>, spawn_as: SaveSettings) -> Self {
        Self(path.into(), spawn_as)
    }

    pub fn str_ref(&self) -> &str {
        self.0.as_ref()
    }

    pub fn save_settings_ref(&self) -> &SaveSettings {
        &self.1
    }
}

impl core::ops::Deref for SpawnSource {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Camera for UI Editor Elements
#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Component, Default, PartialEq)]
#[reflect(Component, Serialize, Deserialize, Default, FromReflect)]
pub struct UICamera;

/// Tag entity with this to hide in node tree editor
#[derive(Component, PartialEq, Eq)]
pub struct TreeHiddenEntity;

/// Internal note for editor use
#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Component, Default, PartialEq)]
#[reflect(Component, Serialize, Deserialize, Default, FromReflect)]
pub struct InternalNote(pub String);

// --------------------------------------------------------------------------------------------

//
// Actual structure of serialized entity data
// Serialized data contains three main parts:
// 1. IdentityData - Contains the name, uuid, and class type of the entity.
// 2. TransformData - Contains the position, rotation, and scale of the entity.
// 3. Reflected components - Contains any additional components that are reflected and serialized.
// 4. Parent - Contains the parent entity UUID if this entity is a child of another entity.

/// Actual Saved Identity data
#[derive(Component, Reflect, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[reflect(Component)]
pub struct IdentityData {
    pub uuid: Uuid,
    pub name: String,
    pub class: GraniteTypes,
}

/// Actual Saved GlobalTransform data
#[derive(Component, Reflect, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[reflect(Component)]
pub struct TransformData {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl TransformData {
    pub fn to_bevy(&self) -> Transform {
        Transform {
            translation: self.position,
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}

// Actual component data is gathered from the ComponentEditor and serialized using bevy reflect

// --------------------------------------------------------------------------------------------

// Main component to flag entities that have granite editor components
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct HasRuntimeData;

// If create material on import true, where should that material name come from?
#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Debug)]
pub enum MaterialNameSource {
    FileName,
    #[default]
    FileContents,
    DefaultMaterial,
    SaveData,
}
impl MaterialNameSource {
    pub fn ui_selectable() -> Vec<Self> {
        vec![
            MaterialNameSource::FileName,
            MaterialNameSource::FileContents,
            MaterialNameSource::DefaultMaterial,
        ]
    }
}

// We send this with the prompt for additional settings on objects that need disk path
#[derive(Debug, Clone, Resource, Serialize, Deserialize, PartialEq)]
pub struct PromptImportSettings {
    pub create_mat_on_import: bool,
    pub material_name_source: MaterialNameSource,
}
impl Default for PromptImportSettings {
    fn default() -> Self {
        Self {
            create_mat_on_import: true,
            material_name_source: MaterialNameSource::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromptData {
    pub file: Option<String>,
    pub import_settings: PromptImportSettings,
}
impl Default for PromptData {
    fn default() -> Self {
        Self {
            file: None,
            import_settings: PromptImportSettings::default(),
        }
    }
}

// Re-exports
pub use component_editor::{
    is_bridge_component_check, BridgeTag, ComponentEditor, ExposedToEditor, ReflectedComponent,
};
pub use deserialize::{deserialize_entities, GraniteEditorSerdeEntity};
pub use editable::{
    Camera3D, DirLight, Empty, GraniteTypes, PointLightData, RectBrush, VolumetricFog, OBJ,
};
pub use generate_tangents::{generate_tangents_system, NeedsTangents};
pub use lifecycle::{
    despawn_entities_by_source_system, despawn_entities_system,
    despawn_recursive_serializable_entities,
};
pub use plugin::EntityPlugin;
pub use serialize::{serialize_entities, EntitySaveReadyData, SceneData, SceneMetadata};

// Im adding this so you cant select the editor camera
// and to stop a crash because you can select a gizmo that then despawns its self
bitflags::bitflags! {
    /// A marker component that an entity should be ignored by the editor
    /// This will be more powerful then not having Bridge
    /// As this is explicitly added to an entity
    #[derive(bevy::ecs::component::Component, Default)]
    pub struct EditorIgnore: usize {
        const GIZMO = 1;
        const PICKING = 2;
        const SERIALIZE = 3;
    }
}
