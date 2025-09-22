use super::data::{FlattenedTreeNode, NodeTreeTabData, RowVisualState};
use bevy::prelude::Entity;
use bevy_egui::egui;
use std::collections::HashMap;

/// Main UI entry point for the node tree tab
pub fn node_tree_tab_ui(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    render_search_bar(ui, data);
    ui.add_space(crate::UI_CONFIG.spacing);
    ui.separator();
    ui.add_space(crate::UI_CONFIG.spacing);

    ui.vertical(|ui| {
        render_virtual_tree(ui, data);
    });
}

/// Renders the search bar and filter controls
fn render_search_bar(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;

    ui.horizontal(|ui| {
        ui.add_space(spacing);
        ui.label("🔍");
        ui.add_space(large_spacing);

        let text_edit_id = egui::Id::new("node_tree_search");
        ui.add(
            egui::TextEdit::singleline(&mut data.search_filter)
                .id(text_edit_id)
                .hint_text("Find entity..."),
        );
        ui.add_space(spacing);
        ui.weak("curated: ");
        ui.checkbox(&mut data.filtered_hierarchy, ())
            .on_hover_ui(|ui| {
                ui.label("Toggle visibility of editor-related entities");
            });
        ui.separator();
        ui.add_space(spacing);
        ui.weak("auto-expand: ");
        ui.checkbox(&mut data.expand_to_enabled, ())
            .on_hover_ui(|ui| {
                ui.label("Auto-expand tree to show selected entities");
            });

        ui.separator();
        ui.add_space(spacing);
        ui.weak("auto-scroll: ");
        ui.checkbox(&mut data.scroll_to_enabled, ())
            .on_hover_ui(|ui| {
                ui.label("Auto-scroll to selected entities");
            });
    });
}

/// Marks the tree cache as dirty, forcing a rebuild on next render
pub fn mark_tree_cache_dirty(data: &mut NodeTreeTabData) {
    data.tree_cache_dirty = true;
}

/// Renders the tree using virtual scrolling for performance
fn render_virtual_tree(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    let search_term = data.search_filter.to_lowercase();

    if search_term.is_empty() {
        render_virtual_hierarchical_tree(ui, data);
    } else {
        render_search_results(ui, data, &search_term);
    }
}

/// Renders the hierarchical tree with virtual scrolling
fn render_virtual_hierarchical_tree(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    if data.tree_cache_dirty {
        rebuild_flattened_tree_cache(data);
        data.tree_cache_dirty = false;
    }

    let available_height = ui.available_height();
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let row_height = ui.fonts(|f| f.row_height(&font_id)) + ui.spacing().button_padding.y * 2.0;

    data.virtual_scroll_state.row_height = row_height;
    data.virtual_scroll_state.total_rows = data.flattened_tree_cache.len();

    if data.virtual_scroll_state.visible_count == 0 {
        data.virtual_scroll_state.visible_count = (available_height / row_height).ceil() as usize;
    }

    if data.flattened_tree_cache.is_empty() {
        ui.label("No entities found");
        return;
    }

    handle_empty_space_drop(ui, data);

    let scroll_area_id = egui::Id::new("node_tree_virtual_scroll");

    egui::ScrollArea::vertical()
        .id_salt(scroll_area_id)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let scroll_offset = ui.clip_rect().min.y - ui.max_rect().min.y;
            let current_scroll = scroll_offset.abs();
            let start_row = (current_scroll / row_height) as usize;
            let buffer = data.virtual_scroll_state.buffer_size;
            let visible_start = start_row.saturating_sub(buffer);
            let visible_end = (start_row + data.virtual_scroll_state.visible_count + buffer * 2)
                .min(data.virtual_scroll_state.total_rows);

            data.virtual_scroll_state.visible_start = visible_start;
            data.virtual_scroll_state.scroll_offset = current_scroll;

            let top_spacing = visible_start as f32 * row_height;
            if top_spacing > 0.0 {
                ui.add_space(top_spacing);
            }

            for i in visible_start..visible_end {
                if let Some(node) = data.flattened_tree_cache.get(i).cloned() {
                    render_virtual_tree_node(ui, &node, data, i);
                }
            }

            let bottom_spacing =
                (data.virtual_scroll_state.total_rows - visible_end) as f32 * row_height;
            if bottom_spacing > 0.0 {
                ui.add_space(bottom_spacing);
            }

            if data.should_scroll_to_selection {
                if let Some(selected_entity) = data.active_selection {
                    if let Some(index) = data
                        .flattened_tree_cache
                        .iter()
                        .position(|node| node.entity == selected_entity)
                    {
                        let target_y = index as f32 * row_height;
                        let target_rect = egui::Rect::from_min_size(
                            egui::pos2(ui.min_rect().min.x, ui.min_rect().min.y + target_y),
                            egui::vec2(ui.available_width(), row_height),
                        );
                        ui.scroll_to_rect(target_rect, Some(egui::Align::Center));
                        data.should_scroll_to_selection = false;
                    }
                }
            }
        });
}

/// Rebuilds the flattened tree cache from the hierarchy
fn rebuild_flattened_tree_cache(data: &mut NodeTreeTabData) {
    let mut new_cache = Vec::new();
    let hierarchy_map = build_hierarchy_map(&data.hierarchy);

    if let Some(root_entities) = hierarchy_map.get(&None) {
        for (entity, name, entity_type) in root_entities {
            flatten_tree_recursive(
                *entity,
                name,
                entity_type,
                &hierarchy_map,
                &data.hierarchy,
                0, // depth
                &mut new_cache,
            );
        }
    }

    data.flattened_tree_cache = new_cache;
}

/// Recursively flattens the tree structure for virtual scrolling
fn flatten_tree_recursive(
    entity: Entity,
    name: &str,
    entity_type: &str,
    hierarchy_map: &HashMap<Option<Entity>, Vec<(Entity, String, String)>>,
    hierarchy: &[super::data::HierarchyEntry],
    depth: usize,
    flattened: &mut Vec<FlattenedTreeNode>,
) {
    // Find the hierarchy entry for this entity
    let hierarchy_entry = hierarchy.iter().find(|entry| entry.entity == entity);
    if let Some(entry) = hierarchy_entry {
        let has_children = hierarchy_map
            .get(&Some(entity))
            .map_or(false, |children| !children.is_empty());

        flattened.push(FlattenedTreeNode {
            entity,
            name: name.to_string(),
            entity_type: entity_type.to_string(),
            parent: entry.parent,
            depth,
            is_expanded: entry.is_expanded,
            has_children,
            is_dummy_parent: entry.is_dummy_parent,
            is_preserve_disk: entry.is_preserve_disk,
            is_preserve_disk_transform: entry.is_preserve_disk_transform,
        });

        // If expanded and has children, recursively add children
        if entry.is_expanded && has_children {
            if let Some(children) = hierarchy_map.get(&Some(entity)) {
                for (child_entity, child_name, child_type) in children {
                    flatten_tree_recursive(
                        *child_entity,
                        child_name,
                        child_type,
                        hierarchy_map,
                        hierarchy,
                        depth + 1,
                        flattened,
                    );
                }
            }
        }
    }
}

/// Renders a single node in the virtual tree
fn render_virtual_tree_node(
    ui: &mut egui::Ui,
    node: &FlattenedTreeNode,
    data: &mut NodeTreeTabData,
    _row_index: usize,
) {
    let visual_state = RowVisualState::from_flattened_node(node, data);

    // Calculate row rect for background
    let available_rect = ui.available_rect_before_wrap();
    let row_height = data.virtual_scroll_state.row_height;
    let row_rect = egui::Rect::from_min_size(
        available_rect.min,
        egui::Vec2::new(available_rect.width(), row_height),
    );

    styling::draw_row_background(ui, &row_rect, &visual_state, "");

    let shift_held = ui.input(|i| i.modifiers.shift);
    let ctrl_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);

    ui.horizontal(|ui| {
        let indent_size = node.depth as f32 * 20.0; // 20px per depth level
        ui.add_space(indent_size);

        let font_id = egui::TextStyle::Button.resolve(ui.style());
        let icon_size = ui.fonts(|f| f.row_height(&font_id));

        // Icon allocation for expand/collapse triangle
        let (icon_rect, icon_response) =
            ui.allocate_exact_size(egui::Vec2::new(icon_size, row_height), egui::Sense::click());

        ui.columns(2, |columns| {
            render_virtual_name_column(
                &mut columns[0],
                node,
                &visual_state,
                data,
                ctrl_held,
                shift_held,
            );

            render_virtual_type_column(
                &mut columns[1],
                node,
                &visual_state,
                !data.filtered_hierarchy,
            );
        });

        // Draw expand/collapse triangle
        styling::draw_expand_triangle(
            ui,
            &icon_rect,
            &icon_response,
            &visual_state,
            "", // No search term in virtual mode
            icon_size,
        );

        // Handle expand/collapse clicks
        if node.has_children && icon_response.clicked() {
            if let Some(entry) = data.hierarchy.iter_mut().find(|e| e.entity == node.entity) {
                entry.is_expanded = !entry.is_expanded;
                data.tree_cache_dirty = true; // Mark cache as dirty
            }
        }

        if node.has_children && icon_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });
}

/// Renders the name column for virtual tree nodes
fn render_virtual_name_column(
    ui: &mut egui::Ui,
    node: &FlattenedTreeNode,
    visual_state: &RowVisualState,
    data: &mut NodeTreeTabData,
    ctrl_held: bool,
    shift_held: bool,
) {
    let (name_text, _type_text) =
        styling::create_highlighted_text(&node.name, &node.entity_type, "", ui);
    let name_button = styling::create_name_button(&name_text, visual_state);
    let button_response = ui.add(name_button);
    let combined_response = ui.interact(
        button_response.rect,
        egui::Id::new(("virtual_tree_node", node.entity)),
        egui::Sense::click_and_drag(),
    );

    super::context_menus::handle_context_menu(ui, node.entity, data, &combined_response);

    if combined_response.clicked() && !visual_state.is_dummy_parent {
        super::selection::handle_selection(node.entity, &node.name, data, ctrl_held, shift_held);
    }

    if !visual_state.is_dummy_parent {
        super::selection::handle_drag_drop(&combined_response, node.entity, data, "");
    }
}

/// Renders the type column for virtual tree nodes
fn render_virtual_type_column(
    ui: &mut egui::Ui,
    node: &FlattenedTreeNode,
    visual_state: &RowVisualState,
    verbose: bool,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if visual_state.is_dummy_parent {
            return;
        }

        if verbose {
            ui.weak(format!("{}", node.entity.index()));
            ui.weak(":");
        }

        ui.label(&node.entity_type);
    });
}

/// Handles dropping entities on empty space (removes parents)
fn handle_empty_space_drop(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    if data.drag_payload.is_some() && ui.input(|i| i.pointer.any_released()) {
        if data.drop_target.is_none() {
            data.drop_target = Some(Entity::PLACEHOLDER);
        }
    }
}

/// Renders search results as a flat list
fn render_search_results(ui: &mut egui::Ui, data: &mut NodeTreeTabData, search_term: &str) {
    let filtered: Vec<_> = data
        .hierarchy
        .iter()
        .filter(|entry| {
            entry.name.to_lowercase().contains(search_term)
                || entry.entity_type.to_lowercase().contains(search_term)
        })
        .cloned()
        .collect();

    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for entry in &filtered {
                render_search_result_node(ui, &entry, data, search_term);
            }

            ui.separator();
            ui.weak(format!("{} results found", filtered.len()));
        });
}

/// Renders a single search result node
fn render_search_result_node(
    ui: &mut egui::Ui,
    entry: &super::data::HierarchyEntry,
    data: &mut NodeTreeTabData,
    search_term: &str,
) {
    let visual_state = RowVisualState::from_hierarchy_entry(entry, data, false);
    let available_rect = ui.available_rect_before_wrap();
    let row_height =
        ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button);
    let row_rect = egui::Rect::from_min_size(
        available_rect.min,
        egui::Vec2::new(available_rect.width(), row_height),
    );

    styling::draw_row_background(ui, &row_rect, &visual_state, search_term);

    let shift_held = ui.input(|i| i.modifiers.shift);
    let ctrl_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);

    ui.horizontal(|ui| {
        ui.columns(2, |columns| {
            render_name_column(
                &mut columns[0],
                &entry.name,
                &entry.entity_type,
                &visual_state,
                search_term,
                data,
                entry.entity,
                ctrl_held,
                shift_held,
            );

            render_type_column(
                &mut columns[1],
                entry.entity,
                &entry.entity_type,
                &visual_state,
                !data.filtered_hierarchy,
            );
        });
    });
}

/// Builds a map of parent -> children for tree rendering
fn build_hierarchy_map(
    hierarchy: &[super::data::HierarchyEntry],
) -> HashMap<Option<Entity>, Vec<(Entity, String, String)>> {
    let mut hierarchy_map: HashMap<Option<Entity>, Vec<(Entity, String, String)>> = HashMap::new();

    for entry in hierarchy {
        let parent = entry.parent;
        let entity_tuple = (entry.entity, entry.name.clone(), entry.entity_type.clone());
        hierarchy_map.entry(parent).or_default().push(entity_tuple);
    }

    hierarchy_map
}

/// Renders the name column (left side)
fn render_name_column(
    ui: &mut egui::Ui,
    name: &str,
    entity_type: &str,
    visual_state: &RowVisualState,
    search_term: &str,
    data: &mut NodeTreeTabData,
    entity: Entity,
    ctrl_held: bool,
    shift_held: bool,
) {
    let (name_text, _type_text) =
        styling::create_highlighted_text(name, entity_type, search_term, ui);
    let name_button = styling::create_name_button(&name_text, visual_state);
    let button_response = ui.add(name_button);
    let combined_response = ui.interact(
        button_response.rect,
        egui::Id::new(("tree_node", entity)),
        egui::Sense::click_and_drag(),
    );
    super::context_menus::handle_context_menu(ui, entity, data, &combined_response);

    // Handle selection clicks (but not for dummy parents)
    if combined_response.clicked() && !visual_state.is_dummy_parent {
        super::selection::handle_selection(entity, name, data, ctrl_held, shift_held);
    }

    // Handle drag and drop (but not for dummy parents)
    if !visual_state.is_dummy_parent {
        super::selection::handle_drag_drop(&combined_response, entity, data, search_term);
    }
}

/// Renders the type column (right side)
fn render_type_column(
    ui: &mut egui::Ui,
    entity: Entity,
    entity_type: &str,
    visual_state: &RowVisualState,
    verbose: bool,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if visual_state.is_dummy_parent {
            return;
        }

        if verbose {
            ui.weak(format!("{}", entity.index()));
            ui.weak(":");
        }

        // Show entity type
        ui.label(entity_type);
    });
}

/// Styling functions for visual elements
pub mod styling {
    use super::*;
    use bevy_egui::egui;

    pub fn draw_row_background(
        ui: &mut egui::Ui,
        row_rect: &egui::Rect,
        visual_state: &RowVisualState,
        search_term: &str,
    ) {
        if visual_state.is_being_dragged {
            let drag_color = ui.style().visuals.selection.bg_fill.gamma_multiply(0.7);
            ui.painter().rect_filled(
                *row_rect,
                ui.style().visuals.menu_corner_radius / 2.,
                drag_color,
            );
        } else if visual_state.is_invalid_drop_target && search_term.is_empty() {
            let error_color = ui.style().visuals.error_fg_color.gamma_multiply(0.3);
            ui.painter().rect_filled(
                *row_rect,
                ui.style().visuals.menu_corner_radius / 2.,
                error_color,
            );
        } else if visual_state.is_valid_drop_target && search_term.is_empty() {
        } else if visual_state.is_active_selected {
            ui.painter().rect_filled(
                *row_rect,
                ui.style().visuals.menu_corner_radius / 2.,
                ui.style().visuals.selection.bg_fill,
            );
        } else if visual_state.is_selected {
            ui.painter().rect_filled(
                *row_rect,
                0.0,
                ui.style().visuals.widgets.inactive.weak_bg_fill,
            );
        }
    }

    /// Draws the expand/collapse triangle
    pub fn draw_expand_triangle(
        ui: &mut egui::Ui,
        icon_rect: &egui::Rect,
        button_response: &egui::Response,
        visual_state: &RowVisualState,
        search_term: &str,
        icon_size: f32,
    ) {
        let text_center_y = button_response.rect.center().y;
        let painter = ui.painter();
        let center = egui::pos2(icon_rect.center().x, text_center_y);
        let half_size = icon_size * 0.3;

        if visual_state.has_children && search_term.is_empty() {
            // Show expand/collapse triangle
            let points = if visual_state.is_expanded {
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

            let triangle_color = get_triangle_color(visual_state, ui);
            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                triangle_color,
                egui::Stroke::NONE,
            ));
        } else if search_term.is_empty() {
            // Show leaf node indicator
            let points = [
                egui::pos2(center.x - half_size, center.y - half_size),
                egui::pos2(center.x + half_size, center.y),
                egui::pos2(center.x - half_size, center.y + half_size),
            ];

            let stroke_color = get_stroke_color(visual_state, ui);
            painter.add(egui::Shape::closed_line(
                points.to_vec(),
                egui::Stroke::new(0.3, stroke_color),
            ));
        }
    }

    /// Creates highlighted text for search results
    pub fn create_highlighted_text(
        name: &str,
        entity_type: &str,
        search_term: &str,
        ui: &egui::Ui,
    ) -> (egui::RichText, egui::RichText) {
        let (highlight_bg, highlight_fg) = if ui.style().visuals.dark_mode {
            (egui::Color32::from_rgb(100, 80, 0), egui::Color32::WHITE)
        } else {
            (egui::Color32::LIGHT_YELLOW, egui::Color32::BLACK)
        };

        let name_text = if !search_term.is_empty() && name.to_lowercase().contains(search_term) {
            egui::RichText::new(name)
                .background_color(highlight_bg)
                .color(highlight_fg)
        } else {
            egui::RichText::new(name)
        };

        let type_text =
            if !search_term.is_empty() && entity_type.to_lowercase().contains(search_term) {
                egui::RichText::new(entity_type)
                    .background_color(highlight_bg)
                    .color(highlight_fg)
            } else {
                egui::RichText::new(entity_type)
            };

        (name_text, type_text)
    }

    /// Creates a styled button for the entity name
    pub fn create_name_button<'a>(
        name_text: &'a egui::RichText,
        visual_state: &RowVisualState,
    ) -> egui::Button<'a> {
        if visual_state.is_dummy_parent {
            create_dummy_parent_button(name_text, visual_state)
        } else if visual_state.is_preserve_disk {
            create_preserve_disk_button(name_text)
        } else if visual_state.is_preserve_disk_transform {
            create_preserve_disk_transform_button(name_text)
        } else {
            create_regular_button(name_text, visual_state)
        }
    }

    /// Creates button for dummy parent (scene file)
    fn create_dummy_parent_button<'a>(
        name_text: &'a egui::RichText,
        visual_state: &RowVisualState,
    ) -> egui::Button<'a> {
        if visual_state.is_active_scene {
            egui::Button::new(
                name_text
                    .clone()
                    .strong()
                    .color(egui::Color32::from_rgb(100, 255, 100)),
            )
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
        } else {
            egui::Button::new(name_text.clone().weak())
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE)
        }
    }

    /// Creates button for PreserveDiskFull entities
    fn create_preserve_disk_button(name_text: &egui::RichText) -> egui::Button<'_> {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "[READ ONLY] ",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(255, 100, 100), // Red
                ..Default::default()
            },
        );
        job.append(&name_text.text(), 0.0, egui::TextFormat::default());

        egui::Button::new(job)
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
    }

    /// Creates button for PreserveDiskTransform entities
    fn create_preserve_disk_transform_button(name_text: &egui::RichText) -> egui::Button<'_> {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "[LIMITED] ",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(255, 255, 100), // Yellow
                ..Default::default()
            },
        );
        job.append(&name_text.text(), 0.0, egui::TextFormat::default());

        egui::Button::new(job)
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE)
    }

    /// Creates regular button for normal entities
    fn create_regular_button<'a>(
        name_text: &'a egui::RichText,
        visual_state: &RowVisualState,
    ) -> egui::Button<'a> {
        if visual_state.is_selected || visual_state.is_active_selected {
            egui::Button::new(name_text.clone().strong())
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE)
        } else {
            egui::Button::new(name_text.clone())
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE)
        }
    }

    /// Gets the appropriate color for expand/collapse triangles
    fn get_triangle_color(visual_state: &RowVisualState, ui: &egui::Ui) -> egui::Color32 {
        if visual_state.is_preserve_disk {
            egui::Color32::from_rgb(200, 120, 120)
        } else if visual_state.is_preserve_disk_transform {
            egui::Color32::from_rgb(200, 170, 80)
        } else {
            ui.style().visuals.text_color()
        }
    }

    /// Gets the appropriate stroke color for leaf node indicators
    fn get_stroke_color(visual_state: &RowVisualState, ui: &egui::Ui) -> egui::Color32 {
        if visual_state.is_preserve_disk {
            egui::Color32::from_rgb(200, 140, 140)
        } else if visual_state.is_preserve_disk_transform {
            egui::Color32::from_rgb(200, 180, 100)
        } else if visual_state.is_selected || visual_state.is_active_selected {
            ui.style().visuals.strong_text_color()
        } else {
            ui.style().visuals.text_color()
        }
    }
}
