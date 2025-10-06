use crate::{
    editor_state::EditorState,
    interface::{
        events::{
            PopupMenuRequestedEvent, RequestCameraEntityFrame, RequestEditorToggle,
            RequestToggleCameraSync, RequestViewportCameraOverride, SetActiveWorld,
        },
        panels::{
            bottom_panel::{BottomDockState, BottomTab},
            right_panel::{SideDockState, SideTab},
        },
        popups::PopupType,
        tabs::{
            debug::ui::DebugTabData, log::LogTabData, EditorSettingsTabData, EntityEditorTabData,
            EventsTabData,
        },
        EditorEvents, NodeTreeTabData,
    },
    viewport::ViewportCameraState,
    UI_CONFIG,
};
use bevy::{ecs::{entity::Entity, system::Commands}, prelude::ResMut};
use bevy_egui::egui;
use bevy_granite_core::{
    absolute_asset_to_rel, entities::SaveSettings, RequestDespawnBySource,
    RequestDespawnSerializableEntities, RequestLoadEvent, RequestSaveEvent, UserInput,
};
use bevy_granite_gizmos::selection::events::EntityEvent;
use native_dialog::FileDialog;

pub fn top_bar_ui(
    side_dock: &mut ResMut<SideDockState>,
    bottom_dock: &mut ResMut<BottomDockState>,
    ui: &mut egui::Ui,
    events: &mut EditorEvents,
    user_input: &UserInput,
    editor_state: &EditorState,
    commands: &mut Commands,
    camera_options: &[(Entity, String)],
    viewport_camera_state: &ViewportCameraState,
) {
    let active_camera_label = if viewport_camera_state.is_using_editor() {
        "Editor Camera".to_string()
    } else {
        camera_options
            .iter()
            .find(|(entity, _)| Some(*entity) == viewport_camera_state.active_override)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| "Unknown Camera".to_string())
    };

    let spacing = UI_CONFIG.spacing;

    ui.vertical(|ui| {
        ui.add_space(spacing);

        // MENUs
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save as").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Granite Scene", &["scene"])
                        .show_save_single_file()
                        .unwrap()
                    {
                        events
                            .save
                            .write(RequestSaveEvent(path.display().to_string()));
                    }
                    ui.close();
                }

                if ui.button("Save (Ctrl + S)").clicked() {
                    let loaded = &editor_state.loaded_sources;
                    if !loaded.is_empty() {
                        for source in loaded.iter() {
                            events.save.write(RequestSaveEvent(source.to_string()));
                        }
                    }
                    ui.close();
                }

                if ui.button("Open (Ctrl + O)").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Granite Scene", &["scene"])
                        .show_open_single_file()
                        .unwrap()
                    {
                        events.load.write(RequestLoadEvent(
                            absolute_asset_to_rel(path.display().to_string()).to_string(),
                            SaveSettings::Runtime,
                            None,
                        ));
                    }
                    ui.close();
                }

                ui.separator();

                ui.menu_button("Despawn", |ui| {
                    if ui.button("Despawn All Entities").clicked() {
                        events.despawn_all.write(RequestDespawnSerializableEntities);
                        ui.close();
                    }

                    ui.separator();

                    ui.label(format!(
                        "Loaded Sources ({}):",
                        editor_state.loaded_sources.len()
                    ));

                    if editor_state.loaded_sources.is_empty() {
                        ui.label("  (No sources loaded)");
                    } else {
                        let sources: Vec<String> =
                            editor_state.loaded_sources.iter().cloned().collect();
                        for source in sources {
                            if ui.button(format!("{}", source)).clicked() {
                                events
                                    .despawn_by_source
                                    .write(RequestDespawnBySource(source));
                                ui.close();
                            }
                        }
                    }
                });

                ui.menu_button("Set Active Scene", |ui| {
                    ui.label(format!(
                        "Available Sources ({}):",
                        editor_state.loaded_sources.len()
                    ));

                    if editor_state.loaded_sources.is_empty() {
                        ui.label("  (No sources loaded)");
                    } else {
                        let sources: Vec<String> =
                            editor_state.loaded_sources.iter().cloned().collect();
                        for source in sources {
                            let is_current = editor_state
                                .current_file
                                .as_ref()
                                .map(|current| current == &source)
                                .unwrap_or(false);

                            let button_text = if is_current {
                                format!("[ACTIVE] {}", source)
                            } else {
                                source.clone()
                            };

                            if ui.button(button_text).clicked() {
                                events.set_active_world.write(SetActiveWorld(source));
                                ui.close();
                            }
                        }
                    }
                });

                ui.separator();

                if ui.button("Open Default World").clicked() {
                    events.load.write(RequestLoadEvent(
                        editor_state.default_world.clone(),
                        SaveSettings::Runtime,
                        None,
                    ));
                    ui.close();
                }

                if ui.button("Save Default World").clicked() {
                    events
                        .save
                        .write(RequestSaveEvent(editor_state.default_world.clone()));

                    ui.close();
                }
            });
            ui.menu_button("Panels", |ui| {
                // Entity Editor
                if !side_dock
                    .dock_state
                    .iter_all_tabs()
                    .any(|(_, tab)| matches!(tab, SideTab::EntityEditor { .. }))
                    && ui.button("EntityEditor").clicked()
                {
                    let tab = SideTab::EntityEditor {
                        data: Box::new(EntityEditorTabData::default()),
                    };
                    side_dock.dock_state.push_to_focused_leaf(tab);
                    ui.close();
                }

                // Node tree
                if !side_dock
                    .dock_state
                    .iter_all_tabs()
                    .any(|(_, tab)| matches!(tab, SideTab::NodeTree { .. }))
                    && ui.button("Entities").clicked()
                {
                    let tab = SideTab::NodeTree {
                        data: Box::new(NodeTreeTabData::default()),
                    };
                    side_dock.dock_state.push_to_focused_leaf(tab);
                    ui.close();
                }

                // Settings
                if !side_dock
                    .dock_state
                    .iter_all_tabs()
                    .any(|(_, tab)| matches!(tab, SideTab::EditorSettings { .. }))
                    && ui.button("Editor Settings").clicked()
                {
                    let tab = SideTab::EditorSettings {
                        data: Box::new(EditorSettingsTabData::default()),
                    };
                    side_dock.dock_state.push_to_focused_leaf(tab);
                    ui.close();
                }

                // Log
                if !bottom_dock
                    .dock_state
                    .iter_all_tabs()
                    .any(|(_, tab)| matches!(tab, BottomTab::Log { .. }))
                    && ui.button("Log").clicked()
                {
                    let tab = BottomTab::Log {
                        data: LogTabData::default(),
                    };
                    bottom_dock.dock_state.push_to_focused_leaf(tab);
                    ui.close();
                }

                // Debug
                if !bottom_dock
                    .dock_state
                    .iter_all_tabs()
                    .any(|(_, tab)| matches!(tab, BottomTab::Debug { .. }))
                    && ui.button("Debug").clicked()
                {
                    let tab = BottomTab::Debug {
                        data: DebugTabData::default(),
                    };
                    bottom_dock.dock_state.push_to_focused_leaf(tab);
                    ui.close();
                }

                // Event
                if !bottom_dock
                    .dock_state
                    .iter_all_tabs()
                    .any(|(_, tab)| matches!(tab, BottomTab::Events { .. }))
                    && ui.button("Events").clicked()
                {
                    let tab = BottomTab::Events {
                        data: EventsTabData::default(),
                    };
                    bottom_dock.dock_state.push_to_focused_leaf(tab);
                    ui.close();
                }
            });
        });

        ui.separator();

        // Buttons
        ui.horizontal(|ui| {
            ui.separator();
            if ui.button("Add Entity (Shft + A) ").clicked() {
                events.popup.write(PopupMenuRequestedEvent {
                    popup: PopupType::AddEntity,
                    mouse_pos: user_input.mouse_pos,
                });
            }
            ui.separator();
            if ui.button("Parents (Shft + P) ").clicked() {
                events.popup.write(PopupMenuRequestedEvent {
                    popup: PopupType::AddRelationship,
                    mouse_pos: user_input.mouse_pos,
                });
            }
            ui.separator();
            if ui.button("Show Help (F1) ").clicked() {
                events.popup.write(PopupMenuRequestedEvent {
                    popup: PopupType::Help,
                    mouse_pos: user_input.mouse_pos,
                });
            }
            ui.separator();
            if ui.button("Toggle Editor (F2) ").clicked() {
                events.toggle_editor.write(RequestEditorToggle);
            }

            ui.separator();
            if ui.button("Toggle Camera Control (F3) ").clicked() {
                events.toggle_cam_sync.write(RequestToggleCameraSync);
            }

            ui.separator();
            ui.label(format!("Viewing: {}", active_camera_label));
            ui.menu_button("Viewport Camera", |ui| {
                let using_editor = viewport_camera_state.is_using_editor();
                if ui
                    .selectable_label(using_editor, "Editor Camera")
                    .clicked()
                    && !using_editor
                {
                    events
                        .viewport_camera
                        .write(RequestViewportCameraOverride { camera: None });
                    ui.close();
                }

                if camera_options.is_empty() {
                    ui.label("No scene cameras targeting the primary window");
                } else {
                    for (entity, label) in camera_options.iter() {
                        let is_active =
                            viewport_camera_state.active_override == Some(*entity);
                        if ui.selectable_label(is_active, label).clicked() && !is_active {
                            events.viewport_camera.write(RequestViewportCameraOverride {
                                camera: Some(*entity),
                            });
                            ui.close();
                        }
                    }
                }
            });

            ui.separator();
            if ui.button("Frame Active (F) ").clicked() {
                events.frame.write(RequestCameraEntityFrame);
            }
            ui.separator();
            if ui.button("Deselect All (U) ").clicked() {
                commands.trigger(EntityEvent::DeselectAll);
            }
            ui.separator();
        });

        ui.add_space(spacing);
    });
}
