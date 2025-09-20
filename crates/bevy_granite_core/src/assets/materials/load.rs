use super::{
    AvailableEditableMaterials, EditableMaterial, EditableMaterialError, EditableMaterialField,
    StandardMaterialDef,
};
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::math::Affine2;
use bevy::prelude::{
    AlphaMode, AssetServer, Assets, Color, Handle, Image, Res, ResMut, StandardMaterial,
};
use bevy::render::render_resource::Face;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

// This was brutal to figure out and I CANNOT believe the is a .load_with_settings() method...
/// Helper function to load textures with REPEAT address mode
pub fn load_texture_with_repeat(asset_server: &AssetServer, path: String) -> Handle<Image> {
    asset_server.load_with_settings(path, |settings: &mut ImageLoaderSettings| {
        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            ..Default::default()
        });
    })
}

/// Creates a EditableMaterial from a definition(wrapper) file and adds it to the asset system
pub fn material_from_path_into_scene(
    path: &str,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    available_materials: &mut ResMut<AvailableEditableMaterials>,
    asset_server: &Res<AssetServer>,
) -> Option<EditableMaterial> {
    if let Some(existing) = available_materials.find_material_by_path(path) {
        //log!(
        //    LogType::Editor,
        //    LogLevel::Info,
        //    LogCategory::Asset,
        //    "Reused existing material: {}",
        //    existing.friendly_name
        //);
        return Some(existing.clone());
    }

    let ron_path = "assets/".to_string() + path;
    let ron = match std::fs::read_to_string(&ron_path) {
        Ok(content) => content,
        Err(e) => {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Entity,
                "Failed to read material file {}: {}",
                ron_path,
                e
            );
            return None;
        }
    };

    let mat_def: StandardMaterialDef = match ron::from_str(&ron) {
        Ok(def) => def,
        Err(e) => {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Entity,
                "Failed to parse material definition from {}: {}",
                ron_path,
                e
            );
            return None;
        }
    };

    let mut found_fields: Vec<EditableMaterialField> = vec![];
    let mut mat = StandardMaterial::default();

    // Base Color
    if let Some(base_color) = mat_def.base_color {
        mat.base_color = Color::srgba(base_color.0, base_color.1, base_color.2, base_color.3);
        found_fields.push(EditableMaterialField::BaseColor);
    }
    if let Some(texture_path) = &mat_def.base_color_texture {
        if !texture_path.is_empty() {
            let handle = load_texture_with_repeat(asset_server, texture_path.clone());
            mat.base_color_texture = Some(handle.clone());
            available_materials
                .image_paths
                .insert(handle, texture_path.clone());
            found_fields.push(EditableMaterialField::BaseColorTexture);
        }
    }

    // Roughness
    if let Some(roughness) = mat_def.roughness {
        mat.perceptual_roughness = roughness;
        found_fields.push(EditableMaterialField::Roughness);
    }
    if let Some(texture_path) = &mat_def.roughness_texture {
        if !texture_path.is_empty() {
            let handle = load_texture_with_repeat(asset_server, texture_path.clone());
            mat.metallic_roughness_texture = Some(handle.clone());
            available_materials
                .image_paths
                .insert(handle, texture_path.clone());
            found_fields.push(EditableMaterialField::RoughnessTexture);
        }
    }

    // Metalness
    if let Some(metalness) = mat_def.metalness {
        mat.metallic = metalness;
        found_fields.push(EditableMaterialField::Metalness);
    }
    if let Some(texture_path) = &mat_def.metalness_texture {
        if !texture_path.is_empty() {
            let handle = load_texture_with_repeat(asset_server, texture_path.clone());
            mat.metallic_roughness_texture = Some(handle.clone());
            available_materials
                .image_paths
                .insert(handle, texture_path.clone());
            found_fields.push(EditableMaterialField::MetalnessTexture);
        }
    }

    // Emissive
    if let Some(emissive) = mat_def.emissive {
        mat.emissive = Color::srgb(emissive.0, emissive.1, emissive.2).into();
        found_fields.push(EditableMaterialField::Emissive);
    }
    if let Some(weight) = mat_def.emissive_exposure_weight {
        mat.emissive_exposure_weight = weight;
        found_fields.push(EditableMaterialField::EmissiveExposureWeight);
    }
    if let Some(texture_path) = &mat_def.emissive_texture {
        if !texture_path.is_empty() {
            let handle = load_texture_with_repeat(asset_server, texture_path.clone());
            mat.emissive_texture = Some(handle.clone());
            available_materials
                .image_paths
                .insert(handle, texture_path.clone());
            found_fields.push(EditableMaterialField::EmissiveTexture);
        }
    }

    // Normal Map
    if let Some(texture_path) = &mat_def.normal_map_texture {
        if !texture_path.is_empty() {
            let handle = load_texture_with_repeat(asset_server, texture_path.clone());
            mat.normal_map_texture = Some(handle.clone());
            available_materials
                .image_paths
                .insert(handle, texture_path.clone());
            found_fields.push(EditableMaterialField::NormalMapTexture);
        }
    }

    // Occlusion Map
    if let Some(texture_path) = &mat_def.occlusion_map {
        if !texture_path.is_empty() {
            let handle = load_texture_with_repeat(asset_server, texture_path.clone());
            mat.occlusion_texture = Some(handle.clone());
            available_materials
                .image_paths
                .insert(handle, texture_path.clone());
            found_fields.push(EditableMaterialField::OcclusionMap);
        }
    }

    // Thickness
    if let Some(thickness) = mat_def.thickness {
        mat.thickness = thickness;
        found_fields.push(EditableMaterialField::Thickness);
    }

    // Attenuation
    if let Some(color) = mat_def.attenuation_color {
        mat.attenuation_color = Color::srgb(color.0, color.1, color.2);
        found_fields.push(EditableMaterialField::AttenuationColor);
    }
    if let Some(distance) = mat_def.attenuation_distance {
        mat.attenuation_distance = distance;
        found_fields.push(EditableMaterialField::AttenuationDistance);
    }

    // Clearcoat
    if let Some(clearcoat) = mat_def.clearcoat {
        mat.clearcoat = clearcoat;
        found_fields.push(EditableMaterialField::Clearcoat);
    }
    if let Some(roughness) = mat_def.clearcoat_perceptual_roughness {
        mat.clearcoat_perceptual_roughness = roughness;
        found_fields.push(EditableMaterialField::ClearcoatPerceptualRoughness);
    }

    // Anisotropy
    if let Some(strength) = mat_def.anisotropy_strength {
        mat.anisotropy_strength = strength;
        found_fields.push(EditableMaterialField::AnisotropyStrength);
    }
    if let Some(rotation) = mat_def.anisotropy_rotation {
        mat.anisotropy_rotation = rotation;
        found_fields.push(EditableMaterialField::AnisotropyRotation);
    }

    // Boolean properties
    if let Some(double_sided) = mat_def.double_sided {
        mat.double_sided = double_sided;
        found_fields.push(EditableMaterialField::DoubleSided);
    }
    if let Some(unlit) = mat_def.unlit {
        mat.unlit = unlit;
        found_fields.push(EditableMaterialField::Unlit);
    }
    if let Some(fog_enabled) = mat_def.fog_enabled {
        mat.fog_enabled = fog_enabled;
        found_fields.push(EditableMaterialField::FogEnabled);
    }

    // Alpha Mode
    if let Some(alpha_mode_str) = &mat_def.alpha_mode {
        mat.alpha_mode = match alpha_mode_str.as_str() {
            "Opaque" => AlphaMode::Opaque,
            "Blend" => AlphaMode::Blend,
            "Mask" => AlphaMode::Mask(0.5), // Default cutoff value
            _ => AlphaMode::Opaque,
        };
        found_fields.push(EditableMaterialField::AlphaMode);
    }

    // Depth Bias
    if let Some(depth_bias) = mat_def.depth_bias {
        mat.depth_bias = depth_bias;
        found_fields.push(EditableMaterialField::DepthBias);
    }

    // Cull Mode
    if let Some(cull_mode_str) = &mat_def.cull_mode {
        mat.cull_mode = match cull_mode_str.as_str() {
            "Front" => Some(Face::Front),
            "Back" => Some(Face::Back),
            "None" => None,
            _ => Some(Face::Back),
        };
        found_fields.push(EditableMaterialField::CullMode);
    }

    // UV Transform
    if let Some(transform_matrix) = &mat_def.uv_transform {
        let uv = [
            transform_matrix[0][0],
            transform_matrix[0][1],
            transform_matrix[1][0],
            transform_matrix[1][1],
            transform_matrix[2][0],
            transform_matrix[2][1],
        ];
        mat.uv_transform = Affine2::from_cols_array(&uv);
        found_fields.push(EditableMaterialField::UvTransform);
    }

    // Create the material handle
    let handle = materials.add(mat);

    // Create the EditableMaterial
    let obj_material = EditableMaterial {
        path: path.to_string(),
        handle: Some(handle),
        def: Some(mat_def.clone()),
        fields: Some(found_fields),
        friendly_name: mat_def.friendly_name.clone(),
        version: 0,
        new_material: false,
        error: EditableMaterialError::None,
        disk_changes: false,
    };

    // Add to available materials
    if let Some(existing) = &mut available_materials.materials {
        if !existing.contains(&obj_material) {
            existing.push(obj_material.clone());
        }
    } else {
        available_materials.materials = Some(vec![obj_material.clone()]);
    }

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Entity,
        "Loaded material: {} with {} fields",
        mat_def.friendly_name,
        obj_material.fields.as_ref().map_or(0, |f| f.len())
    );

    Some(obj_material)
}

/// Creates a vector of EditableMaterial from the given folder path
pub fn materials_from_folder_into_scene(
    folder_path: &str,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    available_materials: &mut ResMut<AvailableEditableMaterials>,
    asset_server: &Res<AssetServer>,
) -> Vec<EditableMaterial> {
    let mut created_materials = Vec::new();
    let assets_folder_path = "assets/".to_string() + folder_path;

    // Recursively collect all .mat files
    let mut ron_files = Vec::new();
    collect_material_files_recursive(&assets_folder_path, &mut ron_files);
    ron_files.sort();

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Entity,
        "Found {} .mat files in folder and subdirectories: {}",
        ron_files.len(),
        folder_path
    );

    for ron_file_path in ron_files {
        if let Some(obj_material) = material_from_path_into_scene(
            &ron_file_path,
            materials,
            available_materials,
            asset_server,
        ) {
            created_materials.push(obj_material);
        }
    }

    log!(
        LogType::Editor,
        LogLevel::OK,
        LogCategory::Asset,
        "Successfully loaded {} materials from: {}",
        created_materials.len(),
        folder_path
    );

    created_materials
}

/// Recursively collects all material .mat files in the given directory and its subdirectories
fn collect_material_files_recursive(current_dir: &str, ron_files: &mut Vec<String>) {
    if !std::path::Path::new(current_dir).exists() {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::System,
            "Directory does not exist, skipping: {}",
            current_dir
        );
        return;
    }

    let dir_entries = match std::fs::read_dir(current_dir) {
        Ok(entries) => entries,
        Err(e) => {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::System,
                "Failed to read directory {}: {}",
                current_dir,
                e
            );
            return;
        }
    };

    for entry in dir_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::System,
                    "Failed to read directory entry: {}",
                    e
                );
                continue;
            }
        };

        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectory
            collect_material_files_recursive(&path.to_string_lossy(), ron_files);
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "mat") {
            // Get the path relative to assets/
            let path_str = path.to_string_lossy();
            if let Some(assets_pos) = path_str.find("assets/") {
                let relative_path = &path_str[assets_pos + 7..]; // Skip "assets/"
                ron_files.push(relative_path.replace('\\', "/")); // Normalize slashes
            }
        }
    }
}

/// Loads a material from a path and returns it if it exists
pub fn get_material_from_path(
    path: &str,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    available_materials: &mut ResMut<AvailableEditableMaterials>,
    asset_server: &Res<AssetServer>,
) -> Option<EditableMaterial> {
    material_from_path_into_scene(path, materials, available_materials, asset_server)
}
