use super::system::{is_descendant_of, is_valid_drop, NodeTreeTabData};
use crate::interface::tabs::node_tree::HierarchyEntry;
use bevy::prelude::Entity;
use bevy_egui::egui;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use std::collections::HashMap;

pub fn node_tree_tab_ui(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    ui.horizontal(|ui| {
        ui.add_space(spacing);
        ui.label("🔍");
        ui.add_space(large_spacing);

        let text_edit_id = egui::Id::new("node_tree_search");
        let _search_response = ui.add(
            egui::TextEdit::singleline(&mut data.search_filter)
                .id(text_edit_id)
                //.desired_width(ui.available_width() - large_spacing)
                .hint_text("Find entity..."),
        );
        ui.add_space(spacing);
        ui.weak("curated: ");
        let _check_response = ui
            .checkbox(&mut data.filtered_hierarchy, ())
            .on_hover_ui(|ui| {
                ui.label("Toggle visibility of editor-related entities");
            });
    });
    ui.add_space(spacing);
    ui.separator();
    ui.add_space(spacing);

    ui.vertical(|ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                display_entity_tree(ui, data);
            });
    });
}

fn draw_node_background(
    ui: &mut egui::Ui,
    row_rect: &egui::Rect,
    entity: Entity,
    data: &NodeTreeTabData,
    is_selected: bool,
    is_active_selected: bool,
    search_term: &str,
) {
    let is_being_dragged = data
        .drag_payload
        .as_ref()
        .map_or(false, |entities| entities.contains(&entity));
    let is_valid_drop_target = data.drag_payload.as_ref().map_or(false, |entities| {
        !entities.contains(&entity) && is_valid_drop(entities, entity, &data.hierarchy)
    });
    let is_invalid_drop_target = data.drag_payload.as_ref().map_or(false, |entities| {
        entities.contains(&entity)
            || entities
                .iter()
                .any(|&dragged_entity| is_descendant_of(entity, dragged_entity, &data.hierarchy))
    });

    if is_being_dragged {
        // Being dragged - use a tinted version of the selection color
        let drag_color = ui.style().visuals.selection.bg_fill.gamma_multiply(0.7);
        ui.painter().rect_filled(
            *row_rect,
            ui.style().visuals.menu_corner_radius / 2.,
            drag_color,
        );
    } else if is_invalid_drop_target && search_term.is_empty() {
        // Invalid drop target - use error color
        let error_color = ui.style().visuals.error_fg_color.gamma_multiply(0.3);
        ui.painter().rect_filled(
            *row_rect,
            ui.style().visuals.menu_corner_radius / 2.,
            error_color,
        );
    } else if is_valid_drop_target && search_term.is_empty() {
    } else if is_active_selected {
        ui.painter().rect_filled(
            *row_rect,
            ui.style().visuals.menu_corner_radius / 2.,
            ui.style().visuals.selection.bg_fill,
        );
    } else if is_selected {
        ui.painter().rect_filled(
            *row_rect,
            0.0,
            ui.style().visuals.widgets.inactive.weak_bg_fill,
        );
    }
}

pub fn display_entity_tree(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    // Handle dropping on empty space to remove parents
    if data.drag_payload.is_some() && ui.input(|i| i.pointer.any_released()) {
        // Check if mouse was released over empty space (no entity hovered)
        if data.drop_target.is_none() {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::UI,
                "Drop on empty space - removing parents"
            );
            // Set a special drop target value to indicate "remove parents"
            data.drop_target = Some(Entity::PLACEHOLDER); // Using placeholder as sentinel value
        }
    }

    let search_term = data.search_filter.to_lowercase();
    if search_term.is_empty() {
        // No search: build tree from real hierarchy, using is_expanded
        let mut hierarchy_map: HashMap<Option<Entity>, Vec<(Entity, String, String)>> =
            HashMap::new();
        for entry in &data.hierarchy {
            let parent = entry.parent;
            let entity_tuple = (entry.entity, entry.name.clone(), entry.entity_type.clone());
            hierarchy_map.entry(parent).or_default().push(entity_tuple);
        }
        if let Some(root_entities) = hierarchy_map.get(&None) {
            for (entity, name, entity_type) in root_entities {
                render_tree_node(
                    ui,
                    *entity,
                    name,
                    entity_type,
                    &hierarchy_map,
                    data,
                    0,
                    &search_term,
                    !data.filtered_hierarchy,
                );
            }
        }
    } else {
        // Search: show flat list of matches, ignore expand/collapse
        let filtered: Vec<HierarchyEntry> = data
            .hierarchy
            .iter()
            .filter(|entry| {
                entry.name.to_lowercase().contains(&search_term)
                    || entry.entity_type.to_lowercase().contains(&search_term)
            })
            .cloned()
            .collect();
        for entry in &filtered {
            // Show as flat list, not tree
            render_tree_node(
                ui,
                entry.entity,
                &entry.name,
                &entry.entity_type,
                &HashMap::new(), // no children in search mode
                data,
                0,
                &search_term,
                !data.filtered_hierarchy,
            );
        }
        ui.separator();
        ui.weak(format!("{} results found", filtered.len()));
    }
}

fn draw_expand_triangle(
    column_ui: &egui::Ui,
    icon_rect: &egui::Rect,
    button_response: &egui::Response,
    visuals: &egui::style::Visuals,
    has_children: bool,
    is_expanded: bool,
    is_selected: bool,
    is_active_selected: bool,
    search_term: &str,
    icon_size: f32,
    is_preserve_disk: bool,
    is_preserve_disk_transform: bool,
) {
    let text_center_y = button_response.rect.center().y;
    let painter = column_ui.painter();
    let center = egui::pos2(icon_rect.center().x, text_center_y);
    let half_size = icon_size * 0.3;

    if has_children && search_term.is_empty() {
        // Only show expand/collapse when not searching
        let points = if is_expanded {
            [
                egui::pos2(center.x - half_size, center.y + half_size),
                egui::pos2(center.x + half_size, center.y - half_size),
                egui::pos2(center.x + half_size, center.y + half_size),
            ]
        } else {
            [
                egui::pos2(center.x - half_size, center.y - half_size),
                egui::pos2(center.x + half_size, center.y),
                egui::pos2(center.x - half_size, center.y + half_size),
            ]
        };

        let triangle_color = if is_preserve_disk {
            egui::Color32::from_rgb(200, 120, 120) // Subtle red for PreserveDiskFull entities
        } else if is_preserve_disk_transform {
            egui::Color32::from_rgb(200, 170, 80) // Yellow for PreserveDiskTransform entities
        } else {
            visuals
                .override_text_color
                .unwrap_or_else(|| column_ui.style().visuals.text_color())
        };
        painter.add(egui::Shape::convex_polygon(
            points.to_vec(),
            triangle_color,
            egui::Stroke::NONE,
        ));
    } else if search_term.is_empty() {
        let points = [
            egui::pos2(center.x - half_size, center.y - half_size),
            egui::pos2(center.x + half_size, center.y),
            egui::pos2(center.x - half_size, center.y + half_size),
        ];

        let stroke_color = if is_preserve_disk {
            egui::Color32::from_rgb(200, 140, 140) // Subtle light red for PreserveDiskFull entity stroke
        } else if is_preserve_disk_transform {
            egui::Color32::from_rgb(200, 180, 100) // Light yellow for PreserveDiskTransform entity stroke
        } else if is_selected || is_active_selected {
            visuals
                .override_text_color
                .unwrap_or_else(|| column_ui.style().visuals.strong_text_color())
        } else if visuals.override_text_color.is_some() {
            let base_color = visuals
                .override_text_color
                .unwrap_or_else(|| column_ui.style().visuals.text_color());
            if base_color.a() < 255 {
                egui::Color32::from_rgb(base_color.r(), base_color.g(), base_color.b())
            // Make solid
            } else {
                base_color
            }
        } else {
            column_ui.style().visuals.text_color()
        };

        painter.add(egui::Shape::closed_line(
            points.to_vec(),
            egui::Stroke::new(0.3, stroke_color),
        ));
    }
}

fn create_highlighted_text(
    name: &str,
    entity_type: &str,
    search_term: &str,
    ui: &egui::Ui,
) -> (egui::RichText, egui::RichText) {
    let (highlight_bg, highlight_fg) = if ui.style().visuals.dark_mode {
        // Dark theme: use a softer yellow background with light text
        (egui::Color32::from_rgb(100, 80, 0), egui::Color32::WHITE)
    } else {
        // Light theme: use bright yellow background with dark text
        (egui::Color32::LIGHT_YELLOW, egui::Color32::BLACK)
    };

    let name_text = if !search_term.is_empty() && name.to_lowercase().contains(search_term) {
        egui::RichText::new(name)
            .background_color(highlight_bg)
            .color(highlight_fg)
    } else {
        egui::RichText::new(name)
    };

    let type_text = if !search_term.is_empty() && entity_type.to_lowercase().contains(search_term) {
        egui::RichText::new(entity_type)
            .background_color(highlight_bg)
            .color(highlight_fg)
    } else {
        egui::RichText::new(entity_type)
    };

    (name_text, type_text)
}

fn create_name_button<'a>(
    name_text: &egui::RichText,
    visuals: &egui::style::Visuals,
    is_selected: bool,
    is_active_selected: bool,
) -> egui::Button<'a> {
    if is_selected || is_active_selected {
        let text_color = visuals
            .override_text_color
            .unwrap_or_else(|| visuals.strong_text_color());
        egui::Button::new(name_text.clone().strong().color(text_color))
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
    } else {
        let text_color = visuals
            .override_text_color
            .unwrap_or_else(|| visuals.text_color());
        egui::Button::new(name_text.clone().color(text_color))
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
    }
}

fn render_tree_node(
    ui: &mut egui::Ui,
    entity: Entity,
    name: &str,
    entity_type: &str,
    hierarchy: &HashMap<Option<Entity>, Vec<(Entity, String, String)>>,
    data: &mut NodeTreeTabData,
    indent_level: usize,
    search_term: &str,
    verbose: bool,
) {
    let spacing = crate::UI_CONFIG.spacing;
    let selected_entity = data.active_selection;
    let is_active_selected = Some(entity) == selected_entity;
    let is_selected = data.selected_entities.contains(&entity);
    let has_children = hierarchy
        .get(&Some(entity))
        .map_or(false, |children| !children.is_empty());

    let hierarchy_entry = data.hierarchy.iter().find(|entry| entry.entity == entity);
    let is_expanded = hierarchy_entry.map_or(false, |entry| entry.is_expanded);
    let is_dummy_parent = hierarchy_entry.map_or(false, |entry| entry.is_dummy_parent);
    let is_preserve_disk = hierarchy_entry.map_or(false, |entry| entry.is_preserve_disk);
    let is_preserve_disk_transform =
        hierarchy_entry.map_or(false, |entry| entry.is_preserve_disk_transform);

    // Pre-allocate space to know the rect size
    let available_rect = ui.available_rect_before_wrap();
    let row_height =
        ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button);
    let row_rect = egui::Rect::from_min_size(
        available_rect.min,
        egui::Vec2::new(available_rect.width(), row_height),
    );

    // Scroll to this item
    if is_active_selected && data.should_scroll_to_selection {
        ui.scroll_to_rect(row_rect, Some(egui::Align::Center));
        data.should_scroll_to_selection = false;
    }

    draw_node_background(
        ui,
        &row_rect,
        entity,
        data,
        is_selected,
        is_active_selected,
        search_term,
    );

    let shift_held = ui.input(|i| i.modifiers.shift);
    let ctrl_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
    let _response = ui.horizontal(|ui| {
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        let icon_size = ui.fonts(|f| f.row_height(&font_id));

        // Icon allocation
        let (icon_rect, icon_response) = ui.allocate_exact_size(
            egui::Vec2::new(
                icon_size,
                ui.spacing().button_padding.y * 2.0
                    + ui.text_style_height(&egui::TextStyle::Button),
            ),
            egui::Sense::click(),
        );

        // Store values we need before entering the columns closure
        let visuals = ui.visuals().clone();
        let style_visuals = ui.style().visuals.clone();

        ui.columns(2, |columns| {
            let (name_text, type_text) =
                create_highlighted_text(name, entity_type, search_term, &columns[0]);

            let name_button = if is_dummy_parent {
                // Check if this dummy parent represents the active scene

                let is_active_scene = data
                    .active_scene_file
                    .as_ref()
                    .map_or(false, |active_file| name == active_file);

                if is_active_scene {
                    // Active scene - create button with mixed colored text
                    let font_id = egui::TextStyle::Button.resolve(&columns[0].style());
                    let mut job = egui::text::LayoutJob::default();
                    job.append(
                        "[ACTIVE] ",
                        0.0,
                        egui::TextFormat {
                            color: egui::Color32::from_rgb(100, 255, 100),
                            font_id: font_id.clone(),
                            ..Default::default()
                        },
                    );
                    job.append(
                        name,
                        0.0,
                        egui::TextFormat {
                            color: visuals
                                .override_text_color
                                .unwrap_or_else(|| style_visuals.text_color()),
                            font_id,
                            ..Default::default()
                        },
                    );
                    egui::Button::new(job)
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                } else {
                    // Inactive spawn source - neutral color
                    let weak_color = visuals
                        .override_text_color
                        .unwrap_or_else(|| style_visuals.text_color());
                    let neutral_text = egui::RichText::new(name).color(weak_color);
                    egui::Button::new(neutral_text)
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE)
                }
            } else if is_preserve_disk {
                // Special styling for PreserveDiskFull entities - red prefix + normal name
                let font_id = egui::TextStyle::Button.resolve(&columns[0].style());
                let mut job = egui::text::LayoutJob::default();
                job.append(
                    "[READ] ",
                    0.0,
                    egui::TextFormat {
                        color: egui::Color32::from_rgb(255, 100, 100),
                        font_id: font_id.clone(),
                        ..Default::default()
                    },
                );
                job.append(
                    name,
                    0.0,
                    egui::TextFormat {
                        color: if is_selected || is_active_selected {
                            visuals
                                .override_text_color
                                .unwrap_or_else(|| visuals.strong_text_color())
                        } else {
                            visuals
                                .override_text_color
                                .unwrap_or_else(|| visuals.text_color())
                        },
                        font_id,
                        ..Default::default()
                    },
                );

                egui::Button::new(job)
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
            } else if is_preserve_disk_transform {
                // Special styling for PreserveDiskTransform entities - yellow prefix + normal name
                let font_id = egui::TextStyle::Button.resolve(&columns[0].style());
                let mut job = egui::text::LayoutJob::default();
                job.append(
                    "[LIMITED] ",
                    0.0,
                    egui::TextFormat {
                        color: egui::Color32::from_rgb(200, 170, 80),
                        font_id: font_id.clone(),
                        ..Default::default()
                    },
                );
                job.append(
                    name,
                    0.0,
                    egui::TextFormat {
                        color: if is_selected || is_active_selected {
                            visuals
                                .override_text_color
                                .unwrap_or_else(|| visuals.strong_text_color())
                        } else {
                            visuals
                                .override_text_color
                                .unwrap_or_else(|| visuals.text_color())
                        },
                        font_id,
                        ..Default::default()
                    },
                );

                egui::Button::new(job)
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE)
            } else {
                // All other entities (including editable SpawnSource ones)
                create_name_button(&name_text, &visuals, is_selected, is_active_selected)
            };

            let button_response = columns[0].add(name_button);

            // Create a combined click and drag interaction over the same area
            let combined_response = columns[0].interact(
                button_response.rect,
                egui::Id::new(format!("interact_{:?}", entity)),
                egui::Sense::click_and_drag(),
            );

            if combined_response.clicked() && !is_dummy_parent {
                handle_node_selection(entity, name, data, ctrl_held, shift_held);
            }

            // Handle drag and drop using the combined_response (but not for dummy parents)
            if !is_dummy_parent {
                handle_drag_and_drop(&combined_response, entity, data, search_term);
            }

            columns[1].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(spacing);

                // Combine entity info and type
                let info_text = if verbose {
                    format!("Entity: {} | {}", entity.index(), entity_type)
                } else {
                    entity_type.to_string()
                };

                if is_preserve_disk {
                    // Special styling for PreserveDiskFull entity type text
                    let red_type_text = egui::RichText::new(&info_text)
                        .color(egui::Color32::from_rgb(255, 150, 150));
                    ui.label(red_type_text);
                } else if is_preserve_disk_transform {
                    // Special styling for PreserveDiskTransform entity type text
                    let yellow_type_text = egui::RichText::new(&info_text)
                        .color(egui::Color32::from_rgb(220, 190, 100));
                    ui.label(yellow_type_text);
                } else if is_selected || is_active_selected {
                    let text_color = visuals
                        .override_text_color
                        .unwrap_or_else(|| style_visuals.text_color());
                    let highlighted_info = if verbose {
                        // For verbose mode, we need to handle highlighting differently
                        if !search_term.is_empty()
                            && entity_type.to_lowercase().contains(search_term)
                        {
                            egui::RichText::new(&info_text)
                                .background_color(if ui.style().visuals.dark_mode {
                                    egui::Color32::from_rgb(100, 80, 0)
                                } else {
                                    egui::Color32::LIGHT_YELLOW
                                })
                                .color(if ui.style().visuals.dark_mode {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::BLACK
                                })
                        } else {
                            egui::RichText::new(&info_text).color(text_color)
                        }
                    } else {
                        type_text.color(text_color)
                    };
                    ui.label(highlighted_info);
                } else {
                    let weak_color = visuals
                        .override_text_color
                        .unwrap_or_else(|| style_visuals.weak_text_color());
                    let highlighted_info = if verbose {
                        // For verbose mode, we need to handle highlighting differently
                        if !search_term.is_empty()
                            && entity_type.to_lowercase().contains(search_term)
                        {
                            egui::RichText::new(&info_text)
                                .background_color(if ui.style().visuals.dark_mode {
                                    egui::Color32::from_rgb(100, 80, 0)
                                } else {
                                    egui::Color32::LIGHT_YELLOW
                                })
                                .color(if ui.style().visuals.dark_mode {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::BLACK
                                })
                        } else {
                            egui::RichText::new(&info_text).color(weak_color)
                        }
                    } else {
                        type_text.color(weak_color)
                    };
                    ui.label(highlighted_info);
                }
            });

            // Draw triangle AFTER both columns
            draw_expand_triangle(
                &columns[0],
                &icon_rect,
                &button_response,
                &visuals,
                has_children,
                is_expanded,
                is_selected,
                is_active_selected,
                search_term,
                icon_size,
                is_preserve_disk,
                is_preserve_disk_transform,
            );

            // Handle icon click for expand/collapse
            if has_children && icon_response.clicked() && search_term.is_empty() {
                if let Some(entry) = data.hierarchy.iter_mut().find(|e| e.entity == entity) {
                    entry.is_expanded = !entry.is_expanded;
                }
            }
        });

        if has_children && icon_response.hovered() && search_term.is_empty() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });

    render_children(
        ui,
        entity,
        hierarchy,
        data,
        has_children,
        is_expanded,
        indent_level,
        search_term,
    );
}

fn render_children(
    ui: &mut egui::Ui,
    entity: Entity,
    hierarchy: &std::collections::HashMap<Option<Entity>, Vec<(Entity, String, String)>>,
    data: &mut NodeTreeTabData,
    has_children: bool,
    is_expanded: bool,
    indent_level: usize,
    search_term: &str,
) {
    // Only show children when not searching and node is expanded
    if has_children && is_expanded && search_term.is_empty() {
        if let Some(children) = hierarchy.get(&Some(entity)) {
            ui.indent("children", |ui| {
                for (child_entity, child_name, child_type) in children {
                    render_tree_node(
                        ui,
                        *child_entity,
                        child_name,
                        child_type,
                        hierarchy,
                        data,
                        indent_level + 1,
                        search_term,
                        !data.filtered_hierarchy,
                    );
                }
            });
        }
    }
}

// Update handle_node_selection to accept ctrl and shift
fn handle_node_selection(
    entity: Entity,
    name: &str,
    data: &mut NodeTreeTabData,
    additive: bool,
    range: bool,
) {
    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::UI,
        "Tree Node Selected: {:?} ('{}') (additive: {}, range: {})",
        entity,
        name,
        additive,
        range
    );
    data.clicked_via_node_tree = true;
    data.new_selection = Some(entity);
    data.additive_selection = additive;
    data.range_selection = range;
}

pub fn expand_to_entity(hierarchy: &mut Vec<HierarchyEntry>, target_entity: Entity) {
    // Find the target
    let mut ancestors = Vec::new();
    let mut current_parent = hierarchy
        .iter()
        .find(|entry| entry.entity == target_entity)
        .and_then(|entry| entry.parent);

    // Walk up
    while let Some(parent_entity) = current_parent {
        ancestors.push(parent_entity);
        current_parent = hierarchy
            .iter()
            .find(|entry| entry.entity == parent_entity)
            .and_then(|entry| entry.parent);
    }

    // Expand
    for ancestor in ancestors {
        if let Some(entry) = hierarchy.iter_mut().find(|e| e.entity == ancestor) {
            entry.is_expanded = true;
        }
    }
}

/// Handle drag and drop functionality for a tree node
fn handle_drag_and_drop(
    response: &egui::Response,
    entity: Entity,
    data: &mut NodeTreeTabData,
    search_term: &str,
) {
    // Only allow drag/drop when not searching
    if !search_term.is_empty() {
        return;
    }

    // Handle drag start
    if response.drag_started() {
        let entities_to_drag = if data.selected_entities.contains(&entity) {
            data.selected_entities.clone()
        } else {
            vec![entity]
        };

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::UI,
            "Drag started: {:?} entities",
            entities_to_drag.len()
        );

        data.drag_payload = Some(entities_to_drag);
    }

    // Handle drop detection when mouse is released
    if data.drag_payload.is_some() && response.ctx.input(|i| i.pointer.any_released()) {
        if response.hovered() {
            // Valid drop target
            if let Some(ref dragged_entities) = data.drag_payload {
                if is_valid_drop(dragged_entities, entity, &data.hierarchy) {
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::UI,
                        "Valid drop: {} entities -> Entity {:?}",
                        dragged_entities.len(),
                        entity
                    );
                    data.drop_target = Some(entity);
                } else {
                    log!(
                        LogType::Editor,
                        LogLevel::Warning,
                        LogCategory::UI,
                        "Invalid drop attempted"
                    );
                }
            }
        }
    }
}
