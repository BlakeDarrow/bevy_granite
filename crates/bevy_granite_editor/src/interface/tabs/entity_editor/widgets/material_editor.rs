use crate::interface::shared::material_selector_combo;
use bevy::pbr::StandardMaterial;
use bevy_egui::egui;
use bevy_granite_core::{
    AvailableEditableMaterials, EditableMaterial, EditableMaterialField, NewEditableMaterial,
    StandardMaterialDef,
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use native_dialog::FileDialog;

pub fn display_add_material_field_dropdown(
    ui: &mut egui::Ui,
    existing_fields: &mut Option<Vec<EditableMaterialField>>,
    material_def: &mut StandardMaterialDef,
) -> bool {
    let mut changed = false;
    let all_fields = EditableMaterialField::all();
    let existing = existing_fields.as_ref().map_or(&[][..], |v| &v[..]);
    let available_fields: Vec<_> = all_fields
        .iter()
        .filter(|f| !existing.contains(f))
        .cloned()
        .collect();

    if !available_fields.is_empty() {
        let width = ui.available_width();
        egui::ComboBox::from_id_salt("add_material_field_dropdown")
            .selected_text("Add field...")
            .width(width)
            .show_ui(ui, |ui| {
                for field in &available_fields {
                    let label = format!("{:?}", field);
                    if ui.button(label).clicked() {
                        if let Some(ref mut fields) = existing_fields {
                            log!(
                                LogType::Editor,
                                LogLevel::OK,
                                LogCategory::Entity,
                                "Added: {:?}",
                                field
                            );
                            fields.push(field.clone());
                        } else {
                            *existing_fields = Some(vec![field.clone()]);
                        }
                        init_default_field(field, material_def);

                        changed = true;
                    }
                }
            });

        ui.end_row();
    }

    changed
}

fn init_default_field(field: &EditableMaterialField, def: &mut StandardMaterialDef) {
    let defaults = StandardMaterial::default();
    let material = def;
    match field {
        EditableMaterialField::Emissive => {
            material.emissive = Some((
                defaults.emissive.red,
                defaults.emissive.green,
                defaults.emissive.blue,
            ));
        }
        EditableMaterialField::BaseColor => {
            let base = defaults.base_color.to_srgba();
            material.base_color = Some((base.red, base.green, base.blue, base.alpha));
        }
        EditableMaterialField::Roughness => {
            material.roughness = Some(defaults.perceptual_roughness);
        }
        EditableMaterialField::RoughnessTexture => {
            material.roughness_texture = Some(String::new());
        }
        EditableMaterialField::Metalness => {
            material.metalness = Some(defaults.metallic);
        }
        EditableMaterialField::MetalnessTexture => {
            material.metalness_texture = Some(String::new());
        }
        EditableMaterialField::BaseColorTexture => {
            material.base_color_texture = Some(String::new());
        }
        EditableMaterialField::EmissiveTexture => {
            material.emissive_texture = Some(String::new());
        }
        EditableMaterialField::EmissiveExposureWeight => {
            material.emissive_exposure_weight = Some(defaults.emissive_exposure_weight);
        }
        //EditableMaterialField::NormalMap => { <- same as normal map texture
        //    material.normal_map = Some(String::new());
        //}
        EditableMaterialField::NormalMapTexture => {
            material.normal_map_texture = Some(String::new());
        }
        EditableMaterialField::OcclusionMap => {
            material.occlusion_map = Some(String::new());
        }
        EditableMaterialField::Thickness => {
            material.thickness = Some(defaults.thickness);
        }
        EditableMaterialField::AttenuationColor => {
            material.attenuation_color = Some((
                defaults.attenuation_color.to_srgba().red,
                defaults.attenuation_color.to_srgba().green,
                defaults.attenuation_color.to_srgba().blue,
            ));
        }
        EditableMaterialField::AttenuationDistance => {
            material.attenuation_distance = Some(defaults.attenuation_distance);
        }
        EditableMaterialField::Clearcoat => {
            material.clearcoat = Some(defaults.clearcoat);
        }
        EditableMaterialField::ClearcoatPerceptualRoughness => {
            material.clearcoat_perceptual_roughness = Some(defaults.clearcoat_perceptual_roughness);
        }
        EditableMaterialField::AnisotropyStrength => {
            material.anisotropy_strength = Some(defaults.anisotropy_strength);
        }
        EditableMaterialField::AnisotropyRotation => {
            material.anisotropy_rotation = Some(defaults.anisotropy_rotation);
        }
        EditableMaterialField::DoubleSided => {
            material.double_sided = Some(defaults.double_sided);
        }
        EditableMaterialField::Unlit => {
            material.unlit = Some(defaults.unlit);
        }
        EditableMaterialField::FogEnabled => {
            material.fog_enabled = Some(defaults.fog_enabled);
        }
        EditableMaterialField::AlphaMode => {
            material.alpha_mode = Some("Opaque".to_string());
        }
        EditableMaterialField::DepthBias => {
            material.depth_bias = Some(defaults.depth_bias);
        }
        EditableMaterialField::CullMode => {
            material.cull_mode = Some("Back".to_string());
        }
        EditableMaterialField::UvTransform => {
            material.uv_transform = Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        }
        _ => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MaterialTab {
    #[default]
    Edit,
    Create,
}

pub fn display_material_settings(ui: &mut egui::Ui, material: &mut EditableMaterial) -> bool {
    let large_spacing = crate::UI_CONFIG.large_spacing;
    let small_spacing = crate::UI_CONFIG.small_spacing;
    let mut changed = false;
    ui.set_width(ui.available_width());
    ui.vertical(|ui| {
        ui.set_width(ui.available_width());
        egui::Grid::new("material_settings_grid")
            .num_columns(2)
            .spacing([large_spacing, small_spacing])
            .striped(true)
            .min_col_width(ui.available_width() / 4.0 - large_spacing * 4.)
            .max_col_width(ui.available_width() / 3.0 - large_spacing * 3.)
            .show(ui, |ui| {
                if let Some(ref mut def) = material.def {
                    let mut temp_value = Some(def.friendly_name.clone());
                    let field_changed =
                        display_text_field(ui, "Name", &mut temp_value, Some(""), false, false);

                    if field_changed {
                        if let Some(new_value) = temp_value {
                            def.friendly_name = new_value.clone();
                            log!(
                                LogType::Editor,
                                LogLevel::Info,
                                LogCategory::Blank,
                                "-----------------------------------------------"
                            );
                            log!(
                                LogType::Editor,
                                LogLevel::Info,
                                LogCategory::UI,
                                "User entered new name: '{}'",
                                def.friendly_name
                            );
                        }
                    }

                    ui.label("Path");
                    ui.label(material.path.to_string());

                    changed = field_changed;
                }
            });

        ui.add_space(large_spacing);
    });
    changed
}

pub fn display_material_edit(ui: &mut egui::Ui, material: &mut EditableMaterial) -> bool {
    let large_spacing = crate::UI_CONFIG.large_spacing;
    let small_spacing = crate::UI_CONFIG.small_spacing;
    let mut changed = false;
    ui.vertical(|ui| {
        ui.set_max_width(ui.available_width());
        egui::Grid::new("material_data_grid")
            .num_columns(3)
            .spacing([large_spacing, small_spacing])
            .striped(true)
            .min_col_width(ui.available_width() / 4.0 - large_spacing * 4.)
            .max_col_width(ui.available_width() / 3.0 - large_spacing * 3.)
            .show(ui, |ui| {
                // ui.spacing_mut().button_padding = egui::Vec2::new(2.0, 2.0);
                if let Some(def) = &mut material.def {
                    if let Some(fields) = material.fields.as_mut() {
                        for field in fields.iter() {
                            changed |= display_standard_material_field(
                                ui,
                                field,
                                def,
                                &StandardMaterial::default(),
                            );
                        }
                    }
                    changed |= display_add_material_field_dropdown(ui, &mut material.fields, def);
                    if changed {
                        material.clean_fields();
                    }
                }
            });

        ui.add_space(large_spacing);
    });
    changed
}

pub fn display_material_creation(ui: &mut egui::Ui, new: &mut NewEditableMaterial) -> (bool, bool) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    let mut changed = false;
    let mut save_clicked = false;
    let mut cancel_clicked = false;

    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Name:");
                ui.add_space(spacing);
                changed |= ui.text_edit_singleline(&mut new.friendly_name).changed();
                ui.add_space(large_spacing);
                ui.label("Directory:");
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    changed |= ui.text_edit_singleline(&mut new.file_dir).changed();

                    ui.spacing_mut().button_padding = egui::Vec2::new(2.0, 2.0);
                    if ui.button("📁").clicked() {
                        let current_dir = std::env::current_dir().unwrap();
                        let assets_dir = current_dir.join("assets");
                        let base_dir = assets_dir.join("materials");

                        if let Some(folder) = FileDialog::new()
                            .set_location(&base_dir)
                            .show_open_single_dir()
                            .unwrap()
                        {
                            let relative_path = folder
                                .strip_prefix(&assets_dir)
                                .map(|p| p.to_string_lossy().replace("\\", "/"))
                                .unwrap_or_else(|_| folder.to_string_lossy().into());

                            new.file_dir = relative_path;
                            changed = true;
                        }
                    }
                });
                ui.add_space(large_spacing);
            });

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    new.create = true;
                    new.rel_path =
                        format!("{}/{}", new.file_dir.trim_end_matches('/'), new.file_name);
                    save_clicked = true;
                }
                ui.add_space(spacing);
                if ui.button("Cancel").clicked() {
                    cancel_clicked = true;
                }
            });
        });
    });

    (save_clicked, cancel_clicked)
}

pub fn display_material_selector_field(
    ui: &mut egui::Ui,
    available_materials: &AvailableEditableMaterials,
    material_builder_open: &mut bool,
    material_search_filter: &mut String,
    class_material_path: &mut String,
    current_material: &mut EditableMaterial,
) -> (bool, bool) {
    let mut changed = false;
    let mut delete_clicked = false;
    let search_filter = material_search_filter;

    ui.vertical(|ui| {
        let combo_response = material_selector_combo(
            ui,
            search_filter,
            available_materials,
            class_material_path,
            current_material,
        );

        ui.separator();
        ui.horizontal(|ui| {
            let create_button = ui.button("Create new");
            if create_button.clicked() {
                *material_builder_open = true;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let delete_button = ui.button("Delete");
                if delete_button.clicked() {
                    delete_clicked = true;
                }
            });
        });

        changed |= combo_response;
    });
    (changed, delete_clicked)
}

// -------------------------------------------------------------------------------------------------------------
// Material Input Types
// -------------------------------------------------------------------------------------------------------------

fn display_slider_field(
    ui: &mut egui::Ui,
    name: &str,
    value: &mut Option<f32>,
    min: f32,
    max: f32,
    default_value: Option<f32>,
) -> bool {
    let mut changed = false;
    if let Some(ref mut val) = value {
        ui.label(name);
        let response = ui.add(
            egui::Slider::new(val, min..=max)
                .step_by(0.01)
                .max_decimals(2),
        );
        if response.changed() {
            changed = true;
        }
        ui.horizontal(|ui| {
            ui.set_max_width(ui.available_width());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").on_hover_text("Clear value").clicked() {
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::UI,
                        "User Removed: {:?}",
                        name
                    );
                    *value = None;
                    changed = true;
                }
                if let Some(default) = default_value {
                    if ui.button("🔄").on_hover_text("Reset to default").clicked() {
                        *value = Some(default);
                        changed = true;
                    }
                }
            });
        });
        ui.end_row();
    }
    changed
}

fn display_drag_field(
    ui: &mut egui::Ui,
    name: &str,
    value: &mut Option<f32>,
    default_value: Option<f32>,
) -> bool {
    let mut changed = false;

    if let Some(ref mut val) = value {
        ui.label(name);
        let response = ui.add(egui::DragValue::new(val).speed(0.01));
        if response.changed() {
            changed = true;
        }

        ui.horizontal(|ui| {
            ui.set_max_width(ui.available_width());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(default) = default_value {
                    if ui.button("❌").on_hover_text("Clear value").clicked() {
                        log!(
                            LogType::Editor,
                            LogLevel::Info,
                            LogCategory::UI,
                            "User Removed: {:?}",
                            name
                        );
                        *value = None;
                        changed = true;
                    }
                    if ui.button("🔄").on_hover_text("Reset to default").clicked() {
                        *value = Some(default);
                        changed = true;
                    }
                }
            });
        });

        ui.end_row();
    }

    changed
}

fn display_vec3_color_field(
    ui: &mut egui::Ui,
    name: &str,
    value: &mut Option<(f32, f32, f32)>,
    default: Option<(f32, f32, f32)>,
) -> bool {
    let mut changed = false;

    let should_clear = if let Some(rgb) = value.as_mut() {
        ui.label(name);

        let mut color = [rgb.0, rgb.1, rgb.2];

        if ui.color_edit_button_rgb(&mut color).changed() {
            *rgb = (color[0], color[1], color[2]);
            changed = true;
        }

        let mut should_clear = false;
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").on_hover_text("Clear value").clicked() {
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::UI,
                        "User Removed: {:?}",
                        name
                    );
                    should_clear = true;
                    changed = true;
                }

                if let Some(default_color) = default {
                    if ui.button("🔄").on_hover_text("Reset to default").clicked() {
                        *rgb = default_color;
                        changed = true;
                    }
                }
            });
        });
        ui.end_row();

        should_clear
    } else {
        false
    };

    if should_clear {
        *value = None;
    }

    changed
}

fn display_color_field(
    ui: &mut egui::Ui,
    name: &str,
    color: &mut Option<(f32, f32, f32, f32)>,
    default_color: Option<(f32, f32, f32, f32)>,
) -> bool {
    let mut changed = false;

    let should_clear = if let Some(color_val) = color.as_mut() {
        ui.label(name);
        let mut egui_color = egui::Color32::from_rgba_premultiplied(
            (color_val.0 * 255.0) as u8,
            (color_val.1 * 255.0) as u8,
            (color_val.2 * 255.0) as u8,
            (color_val.3 * 255.0) as u8,
        );

        if ui.color_edit_button_srgba(&mut egui_color).changed() {
            *color_val = (
                egui_color.r() as f32 / 255.0,
                egui_color.g() as f32 / 255.0,
                egui_color.b() as f32 / 255.0,
                egui_color.a() as f32 / 255.0,
            );
            changed = true;
        }

        let mut should_clear = false;
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").on_hover_text("Clear value").clicked() {
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::UI,
                        "User Removed: {:?}",
                        name
                    );
                    should_clear = true;
                    changed = true;
                }
                if let Some(default) = default_color {
                    if ui.button("🔄").on_hover_text("Reset to default").clicked() {
                        *color_val = default;
                        changed = true;
                    }
                }
            });
        });

        ui.end_row();

        should_clear
    } else {
        false
    };

    // Clear the value outside the borrow scope
    if should_clear {
        *color = None;
    }

    changed
}

fn display_toggle_field(
    ui: &mut egui::Ui,
    name: &str,
    value: &mut Option<bool>,
    default: Option<bool>,
) -> bool {
    let mut changed = false;

    if let Some(ref mut val) = value {
        ui.label(name);

        if ui.checkbox(val, "").changed() {
            changed = true;
        }

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌").on_hover_text("Clear value").clicked() {
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::UI,
                        "User Removed: {:?}",
                        name
                    );
                    *value = None;
                    changed = true;
                }

                if let Some(default_val) = default {
                    if ui.button("🔄").on_hover_text("Reset to default").clicked() {
                        *value = Some(default_val);
                        changed = true;
                    }
                }
            });
        });

        ui.end_row();
    }

    changed
}

fn display_text_field(
    ui: &mut egui::Ui,
    name: &str,
    value: &mut Option<String>,
    default: Option<&str>,
    is_path: bool,
    show_buttons: bool,
) -> bool {
    let mut changed = false;
    let small_spacing = crate::UI_CONFIG.small_spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.label(name);

    if let Some(ref mut val) = value {
        if !is_path {
            if ui.text_edit_singleline(val).changed() {
                changed = true;
            }
        } else {
            ui.horizontal(|ui| {
                ui.set_max_width(ui.available_width() - large_spacing * 3.);

                if ui.text_edit_singleline(val).changed() {
                    changed = true;
                }

                ui.spacing_mut().button_padding = egui::Vec2::new(2.0, 2.0);
                if ui.button("📁").clicked() {
                    let current_dir = std::env::current_dir().unwrap();
                    let assets_dir = current_dir.join("assets");
                    let tex_path = assets_dir.join("textures");

                    // Use textures dir if it exists or can be created, otherwise use current dir
                    let dialog_path =
                        if tex_path.exists() || std::fs::create_dir_all(&tex_path).is_ok() {
                            tex_path
                        } else {
                            current_dir.clone()
                        };

                    if let Ok(Some(path)) = FileDialog::new()
                        .add_filter("Texture Files", &["png", "jpg", "jpeg"])
                        .set_location(&dialog_path)
                        .show_open_single_file()
                    {
                        let relative_path = if let Ok(rel_path) = path.strip_prefix(&assets_dir) {
                            rel_path.to_string_lossy().to_string().replace("\\", "/")
                        } else {
                            path.to_string_lossy().to_string()
                        };
                        *val = relative_path;

                        changed = true;
                    }
                    ui.close();
                }

                ui.add_space(small_spacing);
            });
        }
        if show_buttons {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    //ui.set_max_width(ui.available_width() / 2.0 - 24.);
                    if ui.button("❌").on_hover_text("Clear value").clicked() {
                        log!(
                            LogType::Editor,
                            LogLevel::Info,
                            LogCategory::UI,
                            "User Removed: {:?}",
                            name
                        );
                        *value = None;
                        changed = true;
                    }

                    if let Some(default_val) = default {
                        if ui.button("🔄").on_hover_text("Reset to default").clicked() {
                            *value = Some(default_val.to_string());
                            changed = true;
                        }
                    }
                });
            });
        }
    } else {
        let mut temp = String::new();
        if ui.text_edit_singleline(&mut temp).changed() && !temp.is_empty() {
            *value = Some(temp);
            changed = true;
        }
    }

    ui.end_row();
    changed
}

fn display_uv_scale_field(
    ui: &mut egui::Ui,
    uv_transform: &mut Option<[[f32; 3]; 3]>,
    default: Option<(f32, f32)>,
) -> bool {
    let mut changed = false;

    // Extract current scale from matrix or use default (1.0, 1.0)
    let (mut scale_x, mut scale_y) = if let Some(matrix) = uv_transform {
        (matrix[0][0], matrix[1][1])
    } else if let Some((dx, dy)) = default {
        (dx, dy)
    } else {
        (1.0, 1.0)
    };

    ui.label("UV Scale");
    let mut scale_changed = false;
    ui.horizontal(|ui| {
        scale_changed |= ui
            .add(egui::DragValue::new(&mut scale_x).speed(0.01))
            .changed();
        ui.label("x");
        scale_changed |= ui
            .add(egui::DragValue::new(&mut scale_y).speed(0.01))
            .changed();
        ui.label("y");
    });

    // Buttons row (reset/delete)
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("❌").on_hover_text("Clear value").clicked() {
                *uv_transform = None;
                changed = true;
            }
            if let Some((dx, dy)) = default {
                if ui.button("🔄").on_hover_text("Reset to default").clicked() {
                    *uv_transform = Some([[dx, 0.0, 0.0], [0.0, dy, 0.0], [0.0, 0.0, 1.0]]);
                    changed = true;
                }
            }
        });
    });

    if scale_changed {
        // Only scale, so identity matrix with scale
        *uv_transform = Some([[scale_x, 0.0, 0.0], [0.0, scale_y, 0.0], [0.0, 0.0, 1.0]]);
        changed = true;
    }

    ui.end_row();
    changed
}

// -------------------------------------------------------------------------------------------------------------

pub fn display_standard_material_field(
    ui: &mut egui::Ui,
    field: &EditableMaterialField,
    def: &mut StandardMaterialDef,
    defaults: &StandardMaterial,
) -> bool {
    let mut changed = false;

    match field {
        EditableMaterialField::BaseColor => {
            let default = defaults.base_color.to_srgba();
            changed |= display_color_field(
                ui,
                "Base Color",
                &mut def.base_color,
                Some((default.red, default.green, default.blue, default.alpha)),
            );
        }

        EditableMaterialField::BaseColorTexture => {
            changed |= display_text_field(
                ui,
                "Base Color Texture",
                &mut def.base_color_texture,
                Some(""),
                true,
                true,
            );
        }

        EditableMaterialField::Roughness => {
            changed |= display_slider_field(
                ui,
                "Roughness",
                &mut def.roughness,
                0.0,
                1.0,
                Some(defaults.perceptual_roughness),
            );
        }

        EditableMaterialField::RoughnessTexture => {
            changed |= display_text_field(
                ui,
                "Roughness Texture",
                &mut def.roughness_texture,
                Some(""),
                true,
                true,
            )
        }

        EditableMaterialField::Metalness => {
            changed |= display_slider_field(
                ui,
                "Metalness",
                &mut def.metalness,
                0.0,
                1.0,
                Some(defaults.metallic),
            );
        }

        EditableMaterialField::MetalnessTexture => {
            changed |= display_text_field(
                ui,
                "Metalness Texture",
                &mut def.metalness_texture,
                Some(""),
                true,
                true,
            );
        }

        EditableMaterialField::Emissive => {
            changed |= display_vec3_color_field(
                ui,
                "Emissive",
                &mut def.emissive,
                Some((
                    defaults.emissive.red,
                    defaults.emissive.green,
                    defaults.emissive.blue,
                )),
            );
        }

        EditableMaterialField::EmissiveTexture => {
            changed |= display_text_field(
                ui,
                "Emissive Texture",
                &mut def.emissive_texture,
                Some(""),
                true,
                true,
            );
        }

        EditableMaterialField::EmissiveExposureWeight => {
            changed |= display_slider_field(
                ui,
                "Emissive Exposure Weight",
                &mut def.emissive_exposure_weight,
                0.0,
                10.0,
                Some(defaults.emissive_exposure_weight),
            );
        }

        //EditableMaterialField::NormalMap => { <- same as normal map texture
        //    changed |=
        //        display_text_field(ui, "Normal Map", &mut def.normal_map, Some(""), true, true);
        //}
        EditableMaterialField::NormalMapTexture => {
            changed |= display_text_field(
                ui,
                "Normal Map Texture",
                &mut def.normal_map_texture,
                Some(""),
                true,
                true,
            );
        }

        EditableMaterialField::OcclusionMap => {
            changed |= display_text_field(
                ui,
                "Occlusion Map",
                &mut def.occlusion_map,
                Some(""),
                true,
                true,
            );
        }

        EditableMaterialField::Thickness => {
            changed |= display_drag_field(
                ui,
                "Thickness",
                &mut def.thickness,
                Some(defaults.thickness),
            );
        }

        EditableMaterialField::AttenuationColor => {
            changed |= display_vec3_color_field(
                ui,
                "Attenuation Color",
                &mut def.attenuation_color,
                Some((
                    defaults.attenuation_color.to_srgba().red,
                    defaults.attenuation_color.to_srgba().green,
                    defaults.attenuation_color.to_srgba().blue,
                )),
            );
        }

        EditableMaterialField::AttenuationDistance => {
            changed |= display_drag_field(
                ui,
                "Attenuation Distance",
                &mut def.attenuation_distance,
                Some(f32::INFINITY),
            );
        }

        EditableMaterialField::Clearcoat => {
            changed |= display_slider_field(
                ui,
                "Clearcoat",
                &mut def.clearcoat,
                0.0,
                1.0,
                Some(defaults.clearcoat),
            );
        }

        EditableMaterialField::ClearcoatPerceptualRoughness => {
            changed |= display_slider_field(
                ui,
                "Clearcoat Roughness",
                &mut def.clearcoat_perceptual_roughness,
                0.0,
                1.0,
                Some(defaults.clearcoat_perceptual_roughness),
            );
        }

        EditableMaterialField::AnisotropyStrength => {
            changed |= display_slider_field(
                ui,
                "Anisotropy Strength",
                &mut def.anisotropy_strength,
                0.0,
                1.0,
                Some(defaults.anisotropy_strength),
            );
        }

        EditableMaterialField::AnisotropyRotation => {
            changed |= display_slider_field(
                ui,
                "Anisotropy Rotation",
                &mut def.anisotropy_rotation,
                0.0,
                1.0,
                Some(0.0),
            );
        }

        EditableMaterialField::DoubleSided => {
            changed |= display_toggle_field(
                ui,
                "Double Sided",
                &mut def.double_sided,
                Some(defaults.double_sided),
            );
        }

        EditableMaterialField::Unlit => {
            changed |= display_toggle_field(ui, "Unlit", &mut def.unlit, Some(defaults.unlit));
        }

        EditableMaterialField::FogEnabled => {
            changed |= display_toggle_field(
                ui,
                "Fog Enabled",
                &mut def.fog_enabled,
                Some(defaults.fog_enabled),
            );
        }

        EditableMaterialField::AlphaMode => {
            changed |= display_text_field(
                ui,
                "Alpha Mode",
                &mut def.alpha_mode,
                Some("Blend"),
                false,
                true,
            );
        }

        EditableMaterialField::DepthBias => {
            changed |= display_drag_field(
                ui,
                "Depth Bias",
                &mut def.depth_bias,
                Some(defaults.depth_bias),
            );
        }

        EditableMaterialField::CullMode => {
            changed |= display_text_field(
                ui,
                "Cull Mode",
                &mut def.cull_mode,
                Some("Back"),
                true,
                true,
            );
        }

        EditableMaterialField::UvTransform => {
            changed |= display_uv_scale_field(ui, &mut def.uv_transform, Some((1.0, 1.0)));
        }

        _ => {
            ui.label(format!("{:?} not implemented", field));
            ui.end_row();
        }
    }

    changed
}
