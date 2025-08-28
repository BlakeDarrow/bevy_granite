use bevy::platform::collections::HashMap;
use bevy::prelude::{default, Entity};
use bevy_egui::egui;
use bevy_granite_core::{AvailableEditableMaterials, IdentityData, UserInput};

#[derive(Clone, PartialEq, Default)]
pub struct SelectionInfo {
    pub active_selection: (Option<Entity>, Option<String>),
    pub selection: Option<HashMap<Entity, Option<String>>>,
}

#[derive(Clone, PartialEq, Default)]
pub struct ActiveObjectDetails {
    pub entity: Option<Entity>,
    pub name: Option<String>,
    pub identity_data: Option<IdentityData>,
    pub spawned_from: Option<String>,
}

#[derive(PartialEq, Clone)]
pub struct DebugTabData {
    pub fps_info: (String, String),
    pub selection_info: SelectionInfo,
    pub user_input: UserInput,
    pub current_file: String,
    pub active_object_details: ActiveObjectDetails,
    pub available_materials: AvailableEditableMaterials,
}

impl Default for DebugTabData {
    fn default() -> Self {
        Self {
            current_file: "None".to_string(),
            fps_info: default(),
            user_input: UserInput::default(),
            selection_info: SelectionInfo {
                active_selection: (None, None),
                selection: None,
            },
            available_materials: AvailableEditableMaterials::default(),
            active_object_details: ActiveObjectDetails::default(),
        }
    }
}

pub fn debug_tab_ui(ui: &mut egui::Ui, data: &mut DebugTabData) {
    let spacing = crate::UI_CONFIG.spacing;
    let small_spacing = crate::UI_CONFIG.small_spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.label(data.current_file.clone());
    ui.add_space(small_spacing);
    ui.label(data.fps_info.1.clone());
    //ui.label(data.fps_info.0.clone());
    ui.add_space(spacing);

    ui.collapsing("User Input", |ui| {
        ui.vertical(|ui| {
            ui.label(format!(
                "Current buttons: {:#?}",
                data.user_input.current_button_inputs
            ));
            ui.label(format!(
                "Mouse over egui: {:#?}",
                data.user_input.mouse_over_egui
            ));
        });
    });

    ui.collapsing("Entity Selection", |ui| {
        ui.vertical(|ui| {
            ui.weak("Active:");
            if let Some(active_entity) = data.selection_info.active_selection.0 {
                let active_text = if let Some(active_name) = &data.selection_info.active_selection.1
                {
                    active_name.to_string()
                } else {
                    format!("(Unnamed)  [{}]", active_entity.index())
                };
                ui.label(active_text);
            } else {
                ui.label("None");
            }
            ui.add_space(large_spacing);
            ui.separator();
            ui.add_space(spacing);

            ui.weak("Selection:");
            if let Some(selected_entities) = &data.selection_info.selection {
                if selected_entities.is_empty() {
                    ui.label("None");
                } else {
                    ui.vertical(|ui| {
                        for (entity, name) in selected_entities {
                            let line = match name {
                                Some(n) => n.to_string(),
                                None => format!("(Unnamed)  [{}]", entity.index()),
                            };
                            ui.label(line);
                        }
                    });
                }
            } else {
                ui.label("None");
            }
        });
    });

    ui.collapsing("Active Entity Details", |ui| {
        ui.vertical(|ui| {
            if let Some(entity) = data.active_object_details.entity {
                if let Some(identity_data) = &data.active_object_details.identity_data {
                    ui.collapsing("Identity Data", |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(large_spacing);
                            ui.vertical(|ui| ui.label(format!("{:#?}", identity_data)));
                        });
                    });
                } else {
                    ui.label("No identity data - this is a problem");
                }

                ui.add_space(small_spacing);
                ui.weak("Name:");
                ui.label(
                    data.active_object_details
                        .name
                        .as_deref()
                        .unwrap_or("(Unnamed)"),
                );
                ui.add_space(small_spacing);

                ui.weak("Entity:");
                ui.label(format!("Index: {}", entity.index()));
                ui.add_space(small_spacing);

                ui.weak("UUID:");
                if let Some(identity_data) = &data.active_object_details.identity_data {
                    ui.label(format!("{}", identity_data.uuid));
                } else {
                    ui.label("None");
                }
                ui.add_space(small_spacing);

                ui.weak("Spawned from:");
                if let Some(source) = &data.active_object_details.spawned_from {
                    ui.label(source);
                } else {
                    ui.label("None");
                }
            } else {
                ui.label("No active object selected");
            }
        });
    });

    ui.collapsing("Available Editable Materials", |ui| {
        ui.vertical(|ui| {
            ui.weak("(These are ALL the loaded materials of your project. And on startup, we grab your whole material folder to load.)");
            ui.weak("Editable Materials:");
            ui.label(format!("{:#?}", data.available_materials.materials));
            ui.add_space(small_spacing);
            ui.weak("Image Paths:");
            ui.label(format!("{:#?}", data.available_materials.image_paths));
        });
    });
}
