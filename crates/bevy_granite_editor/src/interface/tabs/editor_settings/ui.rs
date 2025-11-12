use super::{EditorSettingsTabData, SettingsTab};
use crate::{
    interface::{
        layout::SidePanelPosition, tabs::editor_settings::ImportState, themes::ThemeState,
    },
    viewport::ViewportState,
};
use bevy_egui::egui::{self, SliderClamping, UiBuilder};
use bevy_granite_core::MaterialNameSource;

// Helper trait for tracking changes
pub trait ChangeTracker {
    fn mark_changed(&mut self);
}

// Column-based helper functions
pub fn labeled_checkbox_columns(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut bool,
    tooltip: Option<&str>,
) -> bool {
    let prev = *value;
    ui.columns(2, |columns| {
        let label_response = columns[0].label(label);
        if let Some(tooltip_text) = tooltip {
            label_response.on_hover_text(tooltip_text);
        }
        columns[1].checkbox(value, "");
    });
    *value != prev
}

pub fn labeled_color_picker_columns(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut [f32; 3],
    tooltip: Option<&str>,
) -> bool {
    let prev = *color;
    ui.columns(2, |columns| {
        let label_response = columns[0].label(label);
        if let Some(tooltip_text) = tooltip {
            label_response.on_hover_text(tooltip_text);
        }
        columns[1].color_edit_button_rgb(color);
    });
    *color != prev
}

pub fn labeled_color_picker_rgba_columns(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut [f32; 4],
    tooltip: Option<&str>,
) -> bool {
    let prev = *color;
    ui.columns(2, |columns| {
        let label_response = columns[0].label(label);
        if let Some(tooltip_text) = tooltip {
            label_response.on_hover_text(tooltip_text);
        }
        columns[1].color_edit_button_rgba_unmultiplied(color);
    });
    *color != prev
}

pub fn labeled_slider_columns<T>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut T,
    range: std::ops::RangeInclusive<T>,
    step: T,
    decimals: usize,
    suffix: Option<&str>,
    tooltip: Option<&str>,
) -> bool
where
    T: egui::emath::Numeric + PartialEq + Copy,
{
    let prev = *value;
    ui.columns(2, |columns| {
        let label_response = columns[0].label(label);
        if let Some(tooltip_text) = tooltip {
            label_response.on_hover_text(tooltip_text);
        }

        let mut slider = egui::Slider::new(value, range)
            .clamping(SliderClamping::Always)
            .show_value(true)
            .max_decimals(decimals)
            .step_by(step.to_f64());

        if let Some(suffix) = suffix {
            slider = slider.suffix(suffix);
        }

        columns[1].add(slider);
    });
    *value != prev
}

pub fn labeled_combo_columns<T>(
    ui: &mut egui::Ui,
    label: &str,
    selected: &mut T,
    options: &[T],
    id: &str,
    tooltip: Option<&str>,
) -> bool
where
    T: PartialEq + Copy + std::fmt::Debug,
{
    let prev = *selected;
    ui.columns(2, |columns| {
        let label_response = columns[0].label(label);
        if let Some(tooltip_text) = tooltip {
            label_response.on_hover_text(tooltip_text);
        }

        egui::ComboBox::from_id_salt(id)
            .selected_text(format!("{:?}", selected))
            .width(120.0)
            .show_ui(&mut columns[1], |ui| {
                for option in options {
                    ui.selectable_value(selected, *option, format!("{option:?}"));
                }
            });
    });
    *selected != prev
}

// ---------------------------------------------------------------------------------------------------

// Modular section builders

fn build_theme_section(ui: &mut egui::Ui, theme_state: &mut ThemeState) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;

    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.add_space(large_spacing);
            use crate::interface::themes::Theme;
            // Theme selector
            let themes = Theme::all();
            if labeled_combo_columns(
                ui,
                "Theme:",
                &mut theme_state.theme,
                &themes,
                "theme_selector",
                Some("Choose the visual theme for the editor"),
            ) {
                theme_state.theme_changed = true;
            }

            ui.add_space(large_spacing);

            // Font Scale
            if labeled_slider_columns(
                ui,
                "Font Scale:",
                &mut theme_state.font_scale,
                0.5..=2.0,
                0.1,
                1,
                None,
                Some("Scale the UI font size"),
            ) {
                theme_state.font_scale_changed = true;
            }

            ui.add_space(spacing);

            // Spacing
            if labeled_slider_columns(
                ui,
                "Extra Spacing:",
                &mut theme_state.spacing,
                0.0..=10.0,
                1.0,
                0,
                None,
                Some("Add extra spacing between UI elements"),
            ) {
                theme_state.spacing_changed = true;
            }
        });
    });
}

fn build_dock_section(ui: &mut egui::Ui, dock: &mut crate::interface::layout::DockState) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.add_space(spacing);
            if labeled_combo_columns(
                ui,
                "Side Panel Position:",
                &mut dock.side_panel_position,
                &SidePanelPosition::all(),
                "side_panel_position_selector",
                Some("Choose which side the panels appear on"),
            ) {
                dock.changed = true;
            }

            ui.add_space(large_spacing);
            labeled_checkbox_columns(
                ui,
                "Store Tabs on Close:",
                &mut dock.store_position_on_close,
                Some("Remember tab layout when closing and reopening"),
            );
        });
    });
}

fn build_scene_light_section(ui: &mut egui::Ui, scene_light_enabled: &mut bool) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.add_space(large_spacing);
            
            ui.vertical(|ui| {
                ui.label("Scene Light:");
                ui.add_space(spacing);
                
                ui.checkbox(scene_light_enabled, "Enable Scene Light")
                    .on_hover_text("Adds a directional light that follows the editor camera's forward direction. Useful for viewing scenes without lights.");
            });
            
            ui.add_space(spacing);
        });
    });
}

fn build_debug_gizmos_section(ui: &mut egui::Ui, viewport: &mut ViewportState) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.add_space(large_spacing);

            let vis = &mut viewport.visualizers;
            let mut changed = false;

            changed |= labeled_checkbox_columns(
                ui,
                "Debug Gizmos:",
                &mut vis.debug_enabled,
                Some("Enable debug visualization"),
            );

            if vis.debug_enabled {
                ui.indent("debug_gizmos_options", |ui| {
                    ui.add_space(spacing);

                    changed |= labeled_checkbox_columns(
                        ui,
                        "Selection Only:",
                        &mut vis.debug_selected_only,
                        Some("Only show debug gizmos for selected objects"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_checkbox_columns(
                        ui,
                        "Relationship Lines:",
                        &mut vis.debug_relationship_lines,
                        Some("Show lines between Parent/Child relationships"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_color_picker_columns(
                        ui,
                        "Color:",
                        &mut vis.debug_color,
                        Some("Color for debug gizmos"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_slider_columns(
                        ui,
                        "Line Thickness:",
                        &mut vis.debug_line_thickness,
                        0.1..=5.0,
                        0.1,
                        2,
                        None,
                        Some("Thickness of debug gizmo lines"),
                    );
                });
            }

            if changed {
                viewport.changed = true;
            }
        });
    });
}

fn build_debug_icons_section(ui: &mut egui::Ui, viewport: &mut ViewportState) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.add_space(large_spacing);

            let vis = &mut viewport.visualizers;
            let mut changed = false;

            changed |= labeled_checkbox_columns(
                ui,
                "Debug Class Icons:",
                &mut vis.icons_enabled,
                Some("Show debug icons for entities"),
            );

            if vis.icons_enabled {
                ui.indent("debug_icons_options", |ui| {
                    ui.add_space(large_spacing);

                    changed |= labeled_slider_columns(
                        ui,
                        "Icon Size:",
                        &mut vis.icon_size,
                        0.001..=1.0,
                        0.01,
                        3,
                        None,
                        Some("Size of debug icons in world space"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_checkbox_columns(
                        ui,
                        "Distance Scaling:",
                        &mut vis.icon_distance_scaling,
                        Some("Scale icon size based on distance from camera"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_slider_columns(
                        ui,
                        "Max Distance:",
                        &mut vis.icon_max_distance,
                        1.0..=250.0,
                        1.0,
                        0,
                        None,
                        Some("Maximum camera distance to show icons"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_color_picker_columns(
                        ui,
                        "Default Icon Color:",
                        &mut vis.icon_color,
                        Some("Default icon color. This is the color state when icons are not selected"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_checkbox_columns(
                        ui,
                        "Show Active Icons",
                        &mut vis.icon_show_active,
                        Some("Show the entity type icon when actively selected"),
                    );
                    ui.add_space(spacing);
                    changed |= labeled_checkbox_columns(
                        ui,
                        "Show Selected Icons",
                        &mut vis.icon_show_selected,
                        Some("Show the entities type icons when a part of the full selection"),
                    );
                });
            }

            if changed {
                viewport.changed = true;
            }
        });
    });
}

fn build_selection_bounds_section(ui: &mut egui::Ui, viewport: &mut ViewportState) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.add_space(large_spacing);

            let vis = &mut viewport.visualizers;
            let mut changed = false;

            changed |= labeled_checkbox_columns(
                ui,
                "Selection Bounds:",
                &mut vis.selection_enabled,
                Some("Show bounds for selected objects"),
            );

            if vis.selection_enabled {
                ui.indent("selection_bounds_options", |ui| {
                    ui.add_space(large_spacing);

                    changed |= labeled_color_picker_columns(
                        ui,
                        "Active Color:",
                        &mut vis.selection_active_color,
                        Some("Color for active selection bounds"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_color_picker_columns(
                        ui,
                        "Selected Color:",
                        &mut vis.selection_color,
                        Some("Color for non-active selection bounds"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_slider_columns(
                        ui,
                        "Bounds Offset:",
                        &mut vis.selection_bounds_offset,
                        0.0..=1.0,
                        0.01,
                        2,
                        None,
                        Some("Distance to push selection wire bounds outward"),
                    );

                    ui.add_space(spacing);
                    // Special handling for percentage slider
                    let mut percent = vis.selection_corner_length * 100.0;
                    let prev_percent = percent;

                    ui.columns(2, |columns| {
                        columns[0].label("Active Edge Coverage:").on_hover_text(
                            "How much of the active wire bounds should be filled around the edges",
                        );
                        columns[1].add(
                            egui::Slider::new(&mut percent, 0.0..=50.0)
                                .clamping(SliderClamping::Always)
                                .show_value(true)
                                .max_decimals(0)
                                .suffix("%")
                                .step_by(1.0),
                        );
                    });

                    if (percent - prev_percent).abs() > f32::EPSILON {
                        vis.selection_corner_length = percent / 100.0;
                        changed = true;
                    }

                    ui.add_space(spacing);
                    changed |= labeled_slider_columns(
                        ui,
                        "Wire Thickness:",
                        &mut vis.selection_line_thickness,
                        0.1..=15.0,
                        0.1,
                        2,
                        None,
                        Some("Thickness of selection bounds wire"),
                    );
                });
            }

            if changed {
                viewport.changed = true;
            }
        });
    });
}

fn build_grid_section(ui: &mut egui::Ui, viewport: &mut ViewportState) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.add_space(large_spacing);

            let mut changed = false;

            changed |= labeled_checkbox_columns(
                ui,
                "Grid:",
                &mut viewport.grid,
                Some("Show grid overlay"),
            );

            if viewport.grid {
                ui.indent("grid_options", |ui| {
                    ui.add_space(large_spacing);

                    changed |= labeled_slider_columns(
                        ui,
                        "Distance:",
                        &mut viewport.grid_distance,
                        1.0..=400.0,
                        1.0,
                        0,
                        None,
                        Some("How far should the grid be rendered"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_slider_columns(
                        ui,
                        "Size:",
                        &mut viewport.grid_size,
                        0.1..=10.0,
                        0.1,
                        1,
                        None,
                        Some("Size of grid"),
                    );

                    ui.add_space(spacing);
                    changed |= labeled_color_picker_rgba_columns(
                        ui,
                        "Color:",
                        &mut viewport.grid_color,
                        Some("Color of rendered grid"),
                    );
                });
            }

            if changed {
                viewport.changed = true;
            }
        });
    });
}
// only obj right now, so a single section
fn build_import_settings_section(ui: &mut egui::Ui, data: &mut ImportState) {
    let large_spacing = crate::UI_CONFIG.large_spacing;
    let spacing = crate::UI_CONFIG.spacing;
    ui.vertical(|ui| {
        ui.group(|ui| {
            ui.add_space(large_spacing);

            let mut changed = false;

            ui.label("OBJ:");
            ui.indent("obj_options", |ui| {
                ui.add_space(large_spacing);
                changed |= labeled_checkbox_columns(
                    ui,
                    "Apply materials on import",
                    &mut data.import_settings.create_mat_on_import,
                    Some("Should we create scene materials when importing OBJs")
                );

                if data.import_settings.create_mat_on_import {
                    ui.add_space(spacing);
                    changed |= labeled_combo_columns(
                        ui,
                        "Preferred Material Source",
                        &mut data.import_settings.material_name_source,
                        &MaterialNameSource::ui_selectable(),
                        "material_source_import_name",
                        Some("Where should the new material be sourced from? The OBJ file contents ('usemtl'), the OBJ file name, or the engine's default material. If it fails, we use engine default material")
                    );
                };
            });

            if changed {
                data.changed = true;
            }
        });
    });
}

// ---------------------------------------------------------------------------------------------------

// Building the tabs

// Interface tab content
fn build_interface_tab(ui: &mut egui::Ui, data: &mut EditorSettingsTabData) {
    egui::ScrollArea::vertical()
        .auto_shrink([true; 2])
        .show(ui, |ui| {
            build_theme_section(ui, &mut data.theme_state);
            build_dock_section(ui, &mut data.dock);
        });
}

// Viewport tab content
fn build_viewport_tab(ui: &mut egui::Ui, viewport: &mut ViewportState, scene_light_enabled: &mut bool) {
    egui::ScrollArea::vertical()
        .auto_shrink([true; 2])
        .show(ui, |ui| {
            build_scene_light_section(ui, scene_light_enabled);
            build_debug_gizmos_section(ui, viewport);
            build_debug_icons_section(ui, viewport);
            build_selection_bounds_section(ui, viewport);
            build_grid_section(ui, viewport);
        });
}

fn build_import_tab(ui: &mut egui::Ui, data: &mut ImportState) {
    egui::ScrollArea::vertical()
        .auto_shrink([true; 2])
        .show(ui, |ui| {
            build_import_settings_section(ui, data);
        });
}

// ---------------------------------------------------------------------------------------------------

// Main ui

pub fn editor_settings_tab_ui(ui: &mut egui::Ui, data: &mut EditorSettingsTabData) {
    let spacing = crate::UI_CONFIG.spacing;
    let full_rect = ui.available_rect_before_wrap();

    // Reserve space for the button at the bottom
    let button_height =
        ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button);
    let button_spacing = spacing;
    let reserved_bottom_space = button_height + button_spacing;

    ui.scope_builder(UiBuilder::new().max_rect(full_rect), |ui| {
        let content_rect = egui::Rect::from_min_size(
            full_rect.min,
            egui::Vec2::new(
                full_rect.width(),
                full_rect.height() - reserved_bottom_space,
            ),
        );

        ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
            ui.vertical(|ui| {
                // Tab bar
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut data.dock.active_tab,
                        SettingsTab::Viewport,
                        "Viewport",
                    );
                    ui.selectable_value(
                        &mut data.dock.active_tab,
                        SettingsTab::Interface,
                        "Interface",
                    );
                    ui.selectable_value(&mut data.dock.active_tab, SettingsTab::Import, "Import")
                });

                ui.add_space(spacing);

                // Tab content in scroll area
                egui::ScrollArea::vertical().show(ui, |ui| match data.dock.active_tab {
                    SettingsTab::Interface => {
                        build_interface_tab(ui, data);
                    }
                    SettingsTab::Viewport => {
                        build_viewport_tab(ui, &mut data.viewport, &mut data.scene_light_enabled);
                    }
                    SettingsTab::Import => build_import_tab(ui, &mut data.import_state),
                });
            });
        });

        // Button at the very bottom - positioned absolutely
        let button_rect = egui::Rect::from_min_size(
            egui::Pos2::new(full_rect.min.x, full_rect.max.y - button_height),
            egui::Vec2::new(full_rect.width(), button_height),
        );

        ui.scope_builder(UiBuilder::new().max_rect(button_rect), |ui| {
            if ui
                .add_sized(
                    [ui.available_width(), button_height],
                    egui::Button::new("Save Settings"),
                )
                .clicked()
            {
                data.save_requested = true;
            }
        });
    });
}
