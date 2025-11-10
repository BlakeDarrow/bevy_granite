use bevy::{
    app::PostStartup,
    ecs::schedule::IntoScheduleConfigs,
    prelude::{App, Plugin, PreStartup},
};
use bevy_egui::{EguiGlobalSettings, EguiPlugin};
use bevy_granite_logging::setup_logging;
use bevy_obj::ObjPlugin;
use setup::{gather_registered_types, setup_component_editor};

// Sub modules
pub mod assets;
pub mod entities;
pub mod events;
pub mod setup;
pub mod shared;
pub mod world;

// Internal plugins from modules
use crate::assets::AssetPlugin;
use crate::entities::EntityPlugin;
use crate::shared::SharedPlugin;
use crate::world::WorldPlugin;

// Re-exports
pub use assets::{
    get_material_from_path, load_texture_with_repeat, material_from_path_into_scene,
    materials_from_folder_into_scene, AvailableEditableMaterials, EditableMaterial,
    EditableMaterialError, EditableMaterialField, MaterialData, NewEditableMaterial,
    RequiredMaterialData, RequiredMaterialDataMut, StandardMaterialDef,
};
pub use bevy_granite_macros::register_editor_components;

// Marker trait for UI callable events
pub trait UICallableEventMarker {}

// Trait for providing event information to UI
pub trait UICallableEventProvider {
    fn get_struct_name() -> &'static str;
    fn get_event_names() -> &'static [&'static str];
}

pub use entities::{
    BridgeTag, Camera3D, ClassCategory, ComponentEditor, DirLight, EditorIgnore,
    GraniteEditorSerdeEntity, GraniteType, GraniteTypes, HasRuntimeData, IdentityData, MainCamera,
    MaterialNameSource, NeedsTangents, PointLightData, PromptData, PromptImportSettings, RectBrush,
    ReflectedComponent, SaveSettings, SpawnSource, TransformData, TreeHiddenEntity, UICamera,
    VolumetricFog, OBJ,
};
pub use events::{
    CollectRuntimeDataEvent, RequestDespawnBySource, RequestDespawnSerializableEntities,
    RequestLoadEvent, RequestLoadBatchEvent, RequestReloadEvent, RequestSaveEvent, RuntimeDataReadyEvent,
    WorldLoadSuccessEvent, WorldLoadBatchSuccessEvent, WorldSaveSuccessEvent,
};
pub use setup::RegisteredTypeNames;
pub use shared::{
    absolute_asset_to_rel, is_scene_version_compatible, mouse_to_world_delta,
    rel_asset_to_absolute, CursorWindowPos, IconEntity, IconProxy, IconType, InputTypes, UserInput,
};

// Bevy Granite Core plugin
pub struct BevyGraniteCore {
    pub logging: bool,
}
impl Plugin for BevyGraniteCore {
    fn build(&self, app: &mut App) {
        let logging_enabled = self.logging;
        app
            //
            // Plugins (all required)
            //
            // External
            .add_plugins(ObjPlugin)
            .insert_resource(EguiGlobalSettings {
                auto_create_primary_context: false,
                ..Default::default()
            })
            .add_plugins(EguiPlugin::default()) // for UserInput checking if we are over Egui. Ideally a better solution is available as this is the core crate that doest use UI
            // Internal
            .add_plugins(EntityPlugin)
            .add_plugins(WorldPlugin)
            .add_plugins(AssetPlugin)
            .add_plugins(SharedPlugin)
            //
            // Events
            //
            .add_message::<RequestLoadEvent>()
            .add_message::<RequestLoadBatchEvent>()
            .add_message::<WorldLoadSuccessEvent>()
            .add_message::<WorldLoadBatchSuccessEvent>()
            .add_message::<RequestDespawnSerializableEntities>()
            .add_message::<RequestDespawnBySource>()
            .add_message::<WorldSaveSuccessEvent>()
            .add_message::<RequestSaveEvent>()
            .add_message::<CollectRuntimeDataEvent>()
            .add_message::<RuntimeDataReadyEvent>()
            .add_message::<RequestReloadEvent>()
            //
            // Resources
            //
            .insert_resource(RegisteredTypeNames::default())
            //
            // Schedule systems
            //
            .add_systems(PreStartup, setup_logging.run_if(move || logging_enabled))
            .add_systems(PreStartup, gather_registered_types.after(setup_logging))
            .add_systems(PostStartup, setup_component_editor);
    }
}
