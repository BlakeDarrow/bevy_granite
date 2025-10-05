use super::RequestEntityUpdateFromClass;
use crate::{
    entities::{EntitySaveReadyData, PromptData},
    AvailableEditableMaterials, ClassCategory, RequiredMaterialData, RequiredMaterialDataMut,
};
use bevy::{
    asset::{AssetServer, Assets, Handle, RenderAssetUsages},
    ecs::{
        entity::Entity,
        system::{Commands, Res, ResMut},
    },
    mesh::Mesh,
    pbr::StandardMaterial,
    prelude::Image,
    transform::components::Transform,
};
use bevy_egui::egui;
use enum_dispatch::enum_dispatch;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// Central trait that all Granite Editor class types must implement
/// We use enum_dispatch for static polymorphism - i.e. all variants of our enum need same functions available to themselves, and exposed up a level - this saves us a tremendous amount of match arms in our enum
#[enum_dispatch]
pub trait GraniteType {
    /// For identifying a type into a larger group
    /// Mainly useful for UI dropdowns, etc.
    fn category(&self) -> ClassCategory;

    /// Full friendly name of this class
    fn type_name(&self) -> String;

    /// Short name, abv. of this class. Used in UI that shouldn't show full name
    fn type_abv(&self) -> String;

    /// If this type requires user input, like obj file path from a dialog popup
    /// Defaults to false
    fn needs_prompt(&self) -> bool {
        false
    }

    /// If we require user input, this is the config to send to the dialog browser
    /// Base path is first string - relative!
    /// Second list of vectors is file extensions to show in popup
    fn get_prompt_config(&self) -> (String, Vec<&'static str>) {
        ("".to_string(), vec!["*"])
    }

    /// Provide the embedded icon bytes - override this if your type has an icon
    /// Defaults to None - meaning this type has no icon
    fn get_embedded_icon_bytes(&self) -> Option<&'static [u8]> {
        None
    }

    /// Provide the icon filename for error messages - override this if your type has an icon
    /// Defaults to None - meaning this type has no icon
    fn get_icon_filename(&self) -> Option<&'static str> {
        None
    }

    /// Generate icon handle ID from type name
    fn icon_handle(&self) -> Handle<Image> {
        let mut hasher = DefaultHasher::new();
        self.type_name().hash(&mut hasher);
        let lower = hasher.finish() as u128;
        hasher.write("granite_editor_icon".as_bytes());
        let raw = (hasher.finish() as u128) << 64 | lower;
        let id = raw & 0xFFFFFFFFFFFF4FFFBFFFFFFFFFFFFFFF | 0x40008000000000000000; // Make this a valid V4 uuid
        Handle::Uuid(Uuid::from_u128(id), Default::default())
    }

    /// Register embedded icon - default implementation provided, only registers if icon data exists
    /// If your types has an icon, this will be called and register the embedded PNG as bytes
    fn register_embedded_icon(&self, images: &mut ResMut<Assets<Image>>) {
        if let (Some(icon_bytes), Some(filename)) =
            (self.get_embedded_icon_bytes(), self.get_icon_filename())
        {
            let image = Image::from_buffer(
                icon_bytes,
                bevy::image::ImageType::Extension("png"),
                bevy::image::CompressedImageFormats::all(),
                true,
                bevy::image::ImageSampler::Default,
                RenderAssetUsages::RENDER_WORLD,
            )
            .unwrap_or_else(|e| panic!("Failed to load embedded {}: {e}", filename));

            let handle: Handle<Image> = self.icon_handle();
            images.insert(handle.id(), image);
        }
    }

    /// Get icon handle - returns None if no icon
    fn get_icon_handle(&self) -> Option<Handle<Image>> {
        if self.get_embedded_icon_bytes().is_some() {
            Some(self.icon_handle())
        } else {
            None
        }
    }

    /// This is generally used for UI entity creation
    /// Some types require user input like obj file selection
    /// Some types require complex scene data to spawn
    /// Self is actually used to gather data for type spawning
    /// Transform is where to spawn new entity
    /// StandardMaterials is all the scene materials according to Bevy
    /// Meshes is our bevy interface for adding the mesh to the scene
    /// AvailableMaterials is our custom resource that contains our MaterialData and image status'
    /// AssetServer is how we load things like textures into scene
    /// UserSentPath is used if there was a prompt upstream and we pass that string down to load if needed. Useful for asking user to select an .obj file or something
    fn spawn_from_new_identity(
        &mut self,
        commands: &mut Commands,
        transform: Transform,
        standard_materials: ResMut<Assets<StandardMaterial>>,
        meshes: ResMut<Assets<Mesh>>,
        available_materials: ResMut<AvailableEditableMaterials>,
        asset_server: Res<AssetServer>,
        maybe_prompt_data: Option<PromptData>,
    ) -> Entity;

    /// Spawns this type from EntitySaveReady data, generally stored on disk
    /// save_data contains both class and identity, so they don't need self
    /// self only used to match type in this case
    /// SaveData contains the actual disk saved data which includes uuid, transform, identity, components, etc.
    /// StandardMaterials is all the scene materials according to Bevy
    /// Meshes is our bevy interface for adding the mesh to the scene
    /// AvailableMaterials is our custom resource that contains our MaterialData and image status'
    /// AssetServer is how we load things like textures into scene
    fn spawn_from_save_data(
        &self,
        save_data: &EntitySaveReadyData,
        commands: &mut Commands,
        standard_materials: &mut ResMut<Assets<StandardMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
    ) -> Entity;

    /// Called to update the world entity with this class's new data
    /// This gets called when the user updates this ClassData via the UI!
    /// Take a target entity, and the system parameter of available events to send
    fn push_to_entity(&self, entity: Entity, request_update: &mut RequestEntityUpdateFromClass);

    /// Actual editing of self class data via UI
    /// Each class can define how it should be viewed on the panels
    /// If the type has real values inside get_material_data(), we also show a material editor after this in the panel, but that is handled inside the editor plugin
    /// Takes spacing formatted as (small, large, normal)
    fn edit_via_ui(&mut self, ui: &mut egui::Ui, spacing: (f32, f32, f32)) -> bool;

    /// Get this class's required material data
    /// Current Material, Last Material and Path
    fn get_material_data(&self) -> Option<RequiredMaterialData> {
        None
    }

    /// Get this class's required material data as mutable
    /// Current Material, Last Material and Path. We use path as a request basically
    fn get_mut_material_data(&mut self) -> Option<RequiredMaterialDataMut> {
        None
    }

    /// Does this class need a unique handle when duplicating?
    /// Be careful!
    /// Things like a rectangle brush that is editable via the editor - i.e. we can change verts need this to be true, otherwise when we edit one they all will edit. i.e. an instance
    /// DON'T do this for things like obj. That would mean every of that same obj has a unique mesh. NOT ideal. Though it should get corrected on reload?
    fn needs_unique_handle(&self) -> bool {
        false
    }
}
