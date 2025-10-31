use bevy::math::Affine2;
use bevy::prelude::{
    AlphaMode, AssetServer, Assets, Color, Handle, Image, Reflect, Res, ResMut, Resource,
    StandardMaterial,
};
use bevy::render::render_resource::Face;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use ron::ser::to_string_pretty;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::shared::rel_asset_to_absolute;
use crate::{load_texture_with_repeat, material_from_path_into_scene};

// For types that require EditableMaterials, use this struct to hold necessary info
// Path is basically the requestor for brand new entities as the current/last wont exist in a meaningful way
#[derive(Debug)]
pub struct RequiredMaterialData<'a> {
    pub current: &'a EditableMaterial,
    pub last: &'a EditableMaterial,
    pub path: &'a String,
}

#[derive(Debug)]
pub struct RequiredMaterialDataMut<'a> {
    pub current: &'a mut EditableMaterial,
    pub last: &'a mut EditableMaterial,
    pub path: &'a mut String,
}

#[derive(Resource, Default, Clone, PartialEq, Debug)]
pub struct AvailableEditableMaterials {
    pub materials: Option<Vec<EditableMaterial>>,
    pub image_paths: HashMap<Handle<Image>, String>,
}

impl AvailableEditableMaterials {
    pub fn find_material_by_path(&self, path: &str) -> Option<&EditableMaterial> {
        if let Some(existing_materials) = &self.materials {
            existing_materials.iter().find(|m| m.path == path)
        } else {
            None
        }
    }
    pub fn contains_material(&self, material: &EditableMaterial) -> bool {
        if let Some(materials) = &self.materials {
            materials.iter().any(|m| m.path == material.path)
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewEditableMaterial {
    pub file_name: String,
    pub file_dir: String,
    pub friendly_name: String,
    pub rel_path: String,
    pub create: bool,
}

impl Default for NewEditableMaterial {
    fn default() -> Self {
        Self {
            file_dir: "materials/".to_string(),
            file_name: "".to_string(),
            friendly_name: "".to_string(),
            rel_path: "".to_string(),
            create: false,
        }
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Hash, Eq)]
pub enum EditableMaterialField {
    BaseColor,
    BaseColorTexture,
    Roughness,
    Metalness,
    MetallicRoughnessTexture,
    Emissive,
    EmissiveTexture,
    EmissiveExposureWeight,
    //NormalMap,
    NormalMapTexture,
    OcclusionMap,
    Thickness,
    AttenuationColor,
    AttenuationDistance,
    Clearcoat,
    ClearcoatPerceptualRoughness,
    AnisotropyStrength,
    AnisotropyChannel,
    AnisotropyRotation,
    DoubleSided,
    Unlit,
    FogEnabled,
    AlphaMode,
    DepthBias,
    CullMode,
    UvTransform,
}

impl EditableMaterialField {
    pub fn all() -> Vec<EditableMaterialField> {
        use EditableMaterialField::*;
        vec![
            BaseColor,
            BaseColorTexture,
            Roughness,
            Metalness,
            MetallicRoughnessTexture,
            Emissive,
            EmissiveTexture,
            EmissiveExposureWeight,
            //NormalMap,
            NormalMapTexture,
            OcclusionMap,
            Thickness,
            AttenuationColor,
            AttenuationDistance,
            Clearcoat,
            ClearcoatPerceptualRoughness,
            AnisotropyStrength,
            AnisotropyChannel,
            AnisotropyRotation,
            DoubleSided,
            Unlit,
            FogEnabled,
            AlphaMode,
            DepthBias,
            CullMode,
            UvTransform,
        ]
    }
}

#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum EditableMaterialError {
    None,
    PathExists,
}

#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct EditableMaterial {
    pub path: String,
    pub friendly_name: String,
    pub handle: Option<Handle<StandardMaterial>>,
    pub def: Option<StandardMaterialDef>,
    pub fields: Option<Vec<EditableMaterialField>>,
    pub version: u32, // local editor version
    pub new_material: bool,
    pub error: EditableMaterialError,
    pub disk_changes: bool,
}

impl Default for EditableMaterial {
    fn default() -> Self {
        Self {
            path: String::new(),
            friendly_name: String::new(),
            handle: Some(Handle::<StandardMaterial>::default()),
            def: None,
            fields: None,
            version: 0,
            new_material: false,
            error: EditableMaterialError::None,
            disk_changes: false,
        }
    }
}

impl EditableMaterial {
    pub fn set_to_empty(&mut self) {
        self.path = String::new();
        self.friendly_name = String::new();
        self.handle = Some(Handle::<StandardMaterial>::default());
        self.def = None;
        self.fields = None;
        self.version = 0;
        self.new_material = false;
        self.error = EditableMaterialError::None;
        self.disk_changes = false;
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
            || self.friendly_name.is_empty()
            || self.friendly_name == "None"
            || self.friendly_name == "Empty"
    }

    pub fn set_handle(&mut self, handle: Option<Handle<StandardMaterial>>) {
        self.handle = handle.clone()
    }

    pub fn clean_fields(&mut self) {
        if let (Some(def), Some(fields)) = (self.def.as_mut(), self.fields.as_mut()) {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Asset,
                "Fields: {:?}",
                fields
            );

            let mut removed_fields = Vec::new();

            fields.retain(|field| {
                let keep = match field {
                    EditableMaterialField::BaseColor => def.base_color.is_some(),
                    EditableMaterialField::BaseColorTexture => def.base_color_texture.is_some(),
                    EditableMaterialField::Roughness => def.roughness.is_some(),
                    EditableMaterialField::Metalness => def.metalness.is_some(),
                    EditableMaterialField::MetallicRoughnessTexture => def.metallic_roughness_texture.is_some(),
                    EditableMaterialField::Emissive => def.emissive.is_some(),
                    EditableMaterialField::EmissiveTexture => def.emissive_texture.is_some(),
                    EditableMaterialField::EmissiveExposureWeight => {
                        def.emissive_exposure_weight.is_some()
                    }
                    //EditableMaterialField::NormalMap => def.normal_map.is_some(), <- same as normal map texture
                    EditableMaterialField::NormalMapTexture => def.normal_map_texture.is_some(),
                    EditableMaterialField::OcclusionMap => def.occlusion_map.is_some(),
                    EditableMaterialField::Thickness => def.thickness.is_some(),
                    EditableMaterialField::AttenuationColor => def.attenuation_color.is_some(),
                    EditableMaterialField::AttenuationDistance => {
                        def.attenuation_distance.is_some()
                    }
                    EditableMaterialField::Clearcoat => def.clearcoat.is_some(),
                    EditableMaterialField::ClearcoatPerceptualRoughness => {
                        def.clearcoat_perceptual_roughness.is_some()
                    }
                    EditableMaterialField::AnisotropyStrength => def.anisotropy_strength.is_some(),
                    EditableMaterialField::AnisotropyRotation => def.anisotropy_rotation.is_some(),
                    EditableMaterialField::AnisotropyChannel => false, // Not implemented
                    EditableMaterialField::DoubleSided => def.double_sided.is_some(),
                    EditableMaterialField::Unlit => def.unlit.is_some(),
                    EditableMaterialField::FogEnabled => def.fog_enabled.is_some(),
                    EditableMaterialField::AlphaMode => def.alpha_mode.is_some(),
                    EditableMaterialField::DepthBias => def.depth_bias.is_some(),
                    EditableMaterialField::CullMode => def.cull_mode.is_some(),
                    EditableMaterialField::UvTransform => def.uv_transform.is_some(),
                };

                if !keep {
                    removed_fields.push(format!("{:?}", field));

                    match field {
                        EditableMaterialField::BaseColor => def.base_color = None,
                        EditableMaterialField::BaseColorTexture => def.base_color_texture = None,
                        EditableMaterialField::Roughness => def.roughness = None,
                        EditableMaterialField::Metalness => def.metalness = None,
                        EditableMaterialField::MetallicRoughnessTexture => def.metallic_roughness_texture = None,
                        EditableMaterialField::Emissive => def.emissive = None,
                        EditableMaterialField::EmissiveTexture => def.emissive_texture = None,
                        EditableMaterialField::EmissiveExposureWeight => {
                            def.emissive_exposure_weight = None
                        }
                        //EditableMaterialField::NormalMap => def.normal_map = None, <- Same as normal map texture
                        EditableMaterialField::NormalMapTexture => def.normal_map_texture = None,
                        EditableMaterialField::OcclusionMap => def.occlusion_map = None,
                        EditableMaterialField::Thickness => def.thickness = None,
                        EditableMaterialField::AttenuationColor => def.attenuation_color = None,
                        EditableMaterialField::AttenuationDistance => {
                            def.attenuation_distance = None
                        }
                        EditableMaterialField::Clearcoat => def.clearcoat = None,
                        EditableMaterialField::ClearcoatPerceptualRoughness => {
                            def.clearcoat_perceptual_roughness = None
                        }
                        EditableMaterialField::AnisotropyStrength => def.anisotropy_strength = None,
                        EditableMaterialField::AnisotropyRotation => def.anisotropy_rotation = None,
                        EditableMaterialField::AnisotropyChannel => {} // Not implemented
                        EditableMaterialField::DoubleSided => def.double_sided = None,
                        EditableMaterialField::Unlit => def.unlit = None,
                        EditableMaterialField::FogEnabled => def.fog_enabled = None,
                        EditableMaterialField::AlphaMode => def.alpha_mode = None,
                        EditableMaterialField::DepthBias => def.depth_bias = None,
                        EditableMaterialField::CullMode => def.cull_mode = None,
                        EditableMaterialField::UvTransform => def.uv_transform = None,
                    }
                }

                keep
            });

            if !removed_fields.is_empty() {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Asset,
                    "Removed fields: {}",
                    removed_fields.join(", ")
                );
            }
        }
    }

    pub fn save_to_file(&mut self) {
        if let Some(def) = &mut self.def {
            let current_dir = std::env::current_dir().expect("Failed to get current directory");

            if !self.path.starts_with("materials/") {
                log!(
                    LogType::Editor,
                    LogLevel::Error,
                    LogCategory::System,
                    "Cannot save material outside 'assets/materials/': {:?}",
                    self.path
                );

                self.new_material = false;
                return;
            }
            let save_path = current_dir.join("assets").join(&self.path);
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::System,
                "Attempted save path: {:?}",
                save_path
            );

            if self.new_material && Path::new(&save_path).exists() {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::Asset,
                    "Material exists already! Will not create for: {:?}",
                    self.friendly_name
                );

                self.new_material = false;
                self.error = EditableMaterialError::PathExists;
                return;
            }

            let path = Path::new(&save_path);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).expect("Failed to create directories");
                }
            }

            let ron_string = to_string_pretty(def, ron::ser::PrettyConfig::default())
                .expect("Failed to serialize material definition");

            std::fs::write(&save_path, ron_string).expect("Failed to write material file");
            self.new_material = false;
            //self.disk_changes = false;

            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::System,
                "Saved: {:?}",
                save_path
            );
        }
    }

    pub fn reset_errors(&mut self) {
        self.error = EditableMaterialError::None;

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Reset error handling"
        );
    }

    pub fn update_material_handle(
        &mut self,
        def: &StandardMaterialDef,
        materials: &mut Assets<StandardMaterial>,
        available_obj_materials: &mut ResMut<AvailableEditableMaterials>,
        asset_server: &Res<AssetServer>,
    ) {
        let fields = self.fields.get_or_insert_with(Vec::new);
        let defaults = StandardMaterial::default();

        let mut changed = false;

        let old_name = self.friendly_name.clone();
        let new_name = def.friendly_name.clone();
        if old_name != new_name {
            self.friendly_name = def.friendly_name.clone();
            self.disk_changes = true;
            log!(
                LogType::Editor,
                LogLevel::OK,
                LogCategory::Asset,
                "Friendly name changed: '{}'",
                new_name
            );
            changed = true;
        }

        if let Some(handle) = &self.handle {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Asset,
                "Updating materials internal handle..."
            );

            if let Some(existing_material) = materials.get_mut(handle) {
                // Base Color
                if let Some(base_color) = def.base_color {
                    if !fields.contains(&EditableMaterialField::BaseColor) {
                        fields.push(EditableMaterialField::BaseColor);
                    }
                    changed = true;
                    existing_material.base_color =
                        Color::srgba(base_color.0, base_color.1, base_color.2, base_color.3);
                } else {
                    existing_material.base_color = defaults.base_color;
                }

                if let Some(path) = &def.base_color_texture {
                    if !path.is_empty() {
                        let handle = load_texture_with_repeat(asset_server, path.clone(), true);
                        existing_material.base_color_texture = Some(handle.clone());
                        changed = true;
                        if !fields.contains(&EditableMaterialField::BaseColorTexture) {
                            fields.push(EditableMaterialField::BaseColorTexture);
                        }
                        available_obj_materials
                            .image_paths
                            .insert(handle, path.clone());
                    }
                } else {
                    existing_material.base_color_texture = None;
                }

                // Roughness
                if let Some(roughness) = def.roughness {
                    if !fields.contains(&EditableMaterialField::Roughness) {
                        fields.push(EditableMaterialField::Roughness);
                    }

                    changed = true;
                    existing_material.perceptual_roughness = roughness;
                } else {
                    existing_material.perceptual_roughness = defaults.perceptual_roughness;
                }

                // Metalness
                if let Some(metalness) = def.metalness {
                    if !fields.contains(&EditableMaterialField::Metalness) {
                        fields.push(EditableMaterialField::Metalness);
                    }

                    changed = true;
                    existing_material.metallic = metalness;
                } else {
                    existing_material.metallic = defaults.metallic;
                }

                // Metallic Roughness Texture (combined)
                if let Some(path) = &def.metallic_roughness_texture {
                    if !path.is_empty() {
                        if !fields.contains(&EditableMaterialField::MetallicRoughnessTexture) {
                            fields.push(EditableMaterialField::MetallicRoughnessTexture);
                        }
                        let handle = load_texture_with_repeat(asset_server, path.clone(), false);
                        existing_material.metallic_roughness_texture = Some(handle.clone());

                        changed = true;
                        available_obj_materials
                            .image_paths
                            .insert(handle, path.clone());
                    }
                } else {
                    existing_material.metallic_roughness_texture = None;
                }

                // Emissive
                if let Some(emissive) = def.emissive {
                    if !fields.contains(&EditableMaterialField::Emissive) {
                        fields.push(EditableMaterialField::Emissive);
                    }

                    changed = true;
                    existing_material.emissive =
                        Color::srgb(emissive.0, emissive.1, emissive.2).into();
                } else {
                    existing_material.emissive = defaults.emissive;
                }

                if let Some(weight) = def.emissive_exposure_weight {
                    if !fields.contains(&EditableMaterialField::EmissiveExposureWeight) {
                        fields.push(EditableMaterialField::EmissiveExposureWeight);
                    }

                    changed = true;
                    existing_material.emissive_exposure_weight = weight;
                } else {
                    existing_material.emissive_exposure_weight = defaults.emissive_exposure_weight;
                }

                if let Some(path) = &def.emissive_texture {
                    if !path.is_empty() {
                        if !fields.contains(&EditableMaterialField::EmissiveTexture) {
                            fields.push(EditableMaterialField::EmissiveTexture);
                        }
                        let handle = load_texture_with_repeat(asset_server, path.clone(), true);

                        changed = true;
                        existing_material.emissive_texture = Some(handle.clone());
                        available_obj_materials
                            .image_paths
                            .insert(handle, path.clone());
                    }
                } else {
                    existing_material.emissive_texture = None;
                }

                // Normal Map
                if let Some(path) = &def.normal_map_texture {
                    if !path.is_empty() {
                        if !fields.contains(&EditableMaterialField::NormalMapTexture) {
                            fields.push(EditableMaterialField::NormalMapTexture);
                        }
                        let handle = load_texture_with_repeat(asset_server, path.clone(), false);

                        changed = true;
                        existing_material.normal_map_texture = Some(handle.clone());
                        available_obj_materials
                            .image_paths
                            .insert(handle, path.clone());
                    }
                } else {
                    existing_material.normal_map_texture = None;
                }

                // Occlusion Map
                if let Some(path) = &def.occlusion_map {
                    if !path.is_empty() {
                        if !fields.contains(&EditableMaterialField::OcclusionMap) {
                            fields.push(EditableMaterialField::OcclusionMap);
                        }
                        let handle = load_texture_with_repeat(asset_server, path.clone(), false);

                        changed = true;
                        existing_material.occlusion_texture = Some(handle.clone());
                        available_obj_materials
                            .image_paths
                            .insert(handle, path.clone());
                    }
                } else {
                    existing_material.occlusion_texture = None;
                }

                // Thickness
                if let Some(thickness) = def.thickness {
                    if !fields.contains(&EditableMaterialField::Thickness) {
                        fields.push(EditableMaterialField::Thickness);
                    }

                    changed = true;
                    existing_material.thickness = thickness;
                } else {
                    existing_material.thickness = defaults.thickness;
                }

                // Attenuation
                if let Some(color) = def.attenuation_color {
                    if !fields.contains(&EditableMaterialField::AttenuationColor) {
                        fields.push(EditableMaterialField::AttenuationColor);
                    }

                    changed = true;
                    existing_material.attenuation_color = Color::srgb(color.0, color.1, color.2);
                } else {
                    existing_material.attenuation_color = defaults.attenuation_color;
                }

                if let Some(distance) = def.attenuation_distance {
                    if !fields.contains(&EditableMaterialField::AttenuationDistance) {
                        fields.push(EditableMaterialField::AttenuationDistance);
                    }

                    changed = true;
                    existing_material.attenuation_distance = distance;
                } else {
                    existing_material.attenuation_distance = defaults.attenuation_distance;
                }

                // Clearcoat
                if let Some(value) = def.clearcoat {
                    if !fields.contains(&EditableMaterialField::Clearcoat) {
                        fields.push(EditableMaterialField::Clearcoat);
                    }

                    changed = true;
                    existing_material.clearcoat = value;
                } else {
                    existing_material.clearcoat = defaults.clearcoat;
                }

                if let Some(value) = def.clearcoat_perceptual_roughness {
                    if !fields.contains(&EditableMaterialField::ClearcoatPerceptualRoughness) {
                        fields.push(EditableMaterialField::ClearcoatPerceptualRoughness);
                    }

                    changed = true;
                    existing_material.clearcoat_perceptual_roughness = value;
                } else {
                    existing_material.clearcoat_perceptual_roughness =
                        defaults.clearcoat_perceptual_roughness;
                }

                // Anisotropy
                if let Some(value) = def.anisotropy_strength {
                    if !fields.contains(&EditableMaterialField::AnisotropyStrength) {
                        fields.push(EditableMaterialField::AnisotropyStrength);
                    }

                    changed = true;
                    existing_material.anisotropy_strength = value;
                } else {
                    existing_material.anisotropy_strength = defaults.anisotropy_strength;
                }

                if let Some(value) = def.anisotropy_rotation {
                    if !fields.contains(&EditableMaterialField::AnisotropyRotation) {
                        fields.push(EditableMaterialField::AnisotropyRotation);
                    }

                    changed = true;
                    existing_material.anisotropy_rotation = value;
                } else {
                    existing_material.anisotropy_rotation = defaults.anisotropy_rotation;
                }

                // Double-sided
                if let Some(val) = def.double_sided {
                    if !fields.contains(&EditableMaterialField::DoubleSided) {
                        fields.push(EditableMaterialField::DoubleSided);
                    }

                    changed = true;
                    existing_material.double_sided = val;
                } else {
                    existing_material.double_sided = defaults.double_sided;
                }

                // Unlit
                if let Some(val) = def.unlit {
                    if !fields.contains(&EditableMaterialField::Unlit) {
                        fields.push(EditableMaterialField::Unlit);
                    }

                    changed = true;
                    existing_material.unlit = val;
                } else {
                    existing_material.unlit = defaults.unlit;
                }

                // Fog Enabled
                if let Some(val) = def.fog_enabled {
                    if !fields.contains(&EditableMaterialField::FogEnabled) {
                        fields.push(EditableMaterialField::FogEnabled);
                    }

                    changed = true;
                    existing_material.fog_enabled = val;
                } else {
                    existing_material.fog_enabled = defaults.fog_enabled;
                }

                // Alpha Mode
                if let Some(mode_str) = def.alpha_mode.as_deref() {
                    if !fields.contains(&EditableMaterialField::AlphaMode) {
                        fields.push(EditableMaterialField::AlphaMode);
                    }

                    changed = true;
                    existing_material.alpha_mode = match mode_str {
                        "Opaque" => AlphaMode::Opaque,
                        "Blend" => AlphaMode::Blend,
                        _ => existing_material.alpha_mode,
                    };
                } else {
                    existing_material.alpha_mode = defaults.alpha_mode;
                }

                // Depth Bias
                if let Some(bias) = def.depth_bias {
                    if !fields.contains(&EditableMaterialField::DepthBias) {
                        fields.push(EditableMaterialField::DepthBias);
                    }

                    changed = true;
                    existing_material.depth_bias = bias;
                } else {
                    existing_material.depth_bias = defaults.depth_bias;
                }

                // Cull Mode
                if let Some(cull_mode) = def.cull_mode.as_deref() {
                    if !fields.contains(&EditableMaterialField::CullMode) {
                        fields.push(EditableMaterialField::CullMode);
                    }

                    changed = true;
                    existing_material.cull_mode = match cull_mode {
                        "Front" => Some(Face::Front),
                        "Back" => Some(Face::Back),
                        _ => Some(Face::Back),
                    };
                } else {
                    existing_material.cull_mode = defaults.cull_mode;
                }

                // UV Transform
                if let Some(matrix) = &def.uv_transform {
                    if !fields.contains(&EditableMaterialField::UvTransform) {
                        fields.push(EditableMaterialField::UvTransform);
                    }
                    let uv = [
                        matrix[0][0],
                        matrix[0][1],
                        matrix[1][0],
                        matrix[1][1],
                        matrix[2][0],
                        matrix[2][1],
                    ];

                    changed = true;
                    existing_material.uv_transform = Affine2::from_cols_array(&uv);
                } else {
                    existing_material.uv_transform = defaults.uv_transform;
                }

                self.version += 1;
            }

            self.def = Some(def.clone());

            let pre_clean = self.fields.clone();
            self.clean_fields();
            let post_clean = self.fields.clone();

            if pre_clean != post_clean {
                changed = true;
                self.disk_changes = true;
            }

            if self.new_material || self.disk_changes {
                self.save_to_file();
                if self.error == EditableMaterialError::PathExists {
                    return;
                }
            }

            if changed || self.new_material {
                if let Some(materials_vec) = &mut available_obj_materials.materials {
                    if let Some(material_to_update) =
                        materials_vec.iter_mut().find(|mat| mat.path == self.path)
                    {
                        // Only update if the material has actually changed
                        if *material_to_update != *self {
                            *material_to_update = self.clone();
                            log!(
                                LogType::Editor,
                                LogLevel::Info,
                                LogCategory::System,
                                "Updated 'Available Scene Materials' with new material handle",
                            );
                        }
                    }
                }
            }
        }
    }

    /// Check if material exist in the world and scene materials, if not create from name
    pub fn material_exists_and_load(
        &mut self,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
        fallback_name: &str,
        fallback_path: &str,
    ) -> bool {
        let mut saved_new_material = false;

        if fallback_path.is_empty() {
            return false;
        }

        if available_materials
            .find_material_by_path(&self.path)
            .is_none()
        {
            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::Asset,
                "We need a new material during spawn: {}",
                fallback_path
            );

            self.update_name(fallback_name.to_lowercase());
            self.update_path(fallback_path.to_lowercase());
            self.save_to_file();
            saved_new_material = true;
        };

        // Ensure whatever material we have is a part of the scene
        if let Some(material) =
            material_from_path_into_scene(&self.path, materials, available_materials, asset_server)
        {
            *self = material;
        }
        saved_new_material
    }

    //  Engine Default Mesh
    pub fn get_new_unnamed_base_color() -> Self {
        let default_material_def = StandardMaterialDef {
            friendly_name: "".to_string(),
            base_color: Some((1.0, 1.0, 1.0, 1.0)),
            ..Default::default()
        };

        Self {
            path: "".to_string(),
            friendly_name: "".to_string(),
            handle: Some(Handle::<StandardMaterial>::default()),
            def: Some(default_material_def),
            fields: Some(vec![EditableMaterialField::BaseColor]),
            version: 0,
            new_material: false,
            error: EditableMaterialError::None,
            disk_changes: true,
        }
    }

    pub fn update_name(&mut self, name: String) {
        self.friendly_name = name.clone();
        if let Some(def) = &mut self.def {
            def.friendly_name = name.clone();
        }
    }

    pub fn update_path(&mut self, path: String) {
        self.path = path.clone();
    }

    /// Delete this material from disk and remove from available materials list
    pub fn delete_from_disk_and_memory(
        &self,
        available_materials: &mut ResMut<AvailableEditableMaterials>,
    ) -> bool {
        // Don't allow deleting the "None" material
        if self.friendly_name == "None" || self.is_empty() {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::Asset,
                "Cannot delete the None material or empty material"
            );
            return false;
        }

        let mut success = true;

        // Try to delete the file from disk
        if !self.path.is_empty() {
            let abs_path = rel_asset_to_absolute(&self.path);
            let file_path = abs_path.to_string();

            if std::fs::metadata(&file_path).is_ok() {
                match std::fs::remove_file(&file_path) {
                    Ok(_) => {
                        log!(
                            LogType::Editor,
                            LogLevel::OK,
                            LogCategory::Asset,
                            "Successfully deleted material file: {}",
                            self.path
                        );
                    }
                    Err(e) => {
                        log!(
                            LogType::Editor,
                            LogLevel::Error,
                            LogCategory::Asset,
                            "Failed to delete material file '{}': {} (file may be read-only or locked)",
                            self.path,
                            e
                        );
                        success = false;
                    }
                }
            } else {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::Asset,
                    "Material file not found on disk: {}",
                    self.path
                );
            }
        }

        // Remove from available materials list (even if file deletion failed)
        if let Some(materials) = &mut available_materials.materials {
            materials.retain(|m| m.path != self.path);
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Asset,
                "Removed material '{}' from available materials list",
                self.friendly_name
            );
        }

        success
    }
}

#[derive(Reflect, Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct StandardMaterialDef {
    pub friendly_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color: Option<(f32, f32, f32, f32)>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metalness: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive: Option<(f32, f32, f32)>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_exposure_weight: Option<f32>,

    //#[serde(skip_serializing_if = "Option::is_none")] <- same as normal map
    //pub normal_map: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occlusion_map: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thickness: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attenuation_color: Option<(f32, f32, f32)>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attenuation_distance: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub clearcoat: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub clearcoat_perceptual_roughness: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub anisotropy_strength: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub anisotropy_rotation: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_sided: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlit: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha_mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth_bias: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cull_mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_transform: Option<[[f32; 3]; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_color_texture: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic_roughness_texture: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub emissive_texture: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal_map_texture: Option<String>,
}

impl Default for StandardMaterialDef {
    fn default() -> Self {
        Self {
            friendly_name: "None".to_string(),
            base_color: None,
            roughness: None,
            metalness: None,
            emissive: None,
            emissive_exposure_weight: None,
            //normal_map: None, <- same as normal map texture
            occlusion_map: None,
            thickness: None,
            attenuation_color: None,
            attenuation_distance: None,
            clearcoat: None,
            clearcoat_perceptual_roughness: None,
            anisotropy_strength: None,
            anisotropy_rotation: None,
            double_sided: None,
            unlit: None,
            fog_enabled: None,
            alpha_mode: None,
            depth_bias: None,
            cull_mode: None,
            uv_transform: None,
            base_color_texture: None,
            metallic_roughness_texture: None,
            emissive_texture: None,
            normal_map_texture: None,
        }
    }
}
