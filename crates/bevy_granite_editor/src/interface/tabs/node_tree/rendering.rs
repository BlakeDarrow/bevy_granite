use super::data::{NodeTreeTabData, RowVisualState};
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
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                render_entity_tree(ui, data);
            });
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
    });
}

/// Renders the main entity tree
fn render_entity_tree(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    handle_empty_space_drop(ui, data);

    let search_term = data.search_filter.to_lowercase();

    if search_term.is_empty() {
        render_hierarchical_tree(ui, data, &search_term);
    } else {
        render_search_results(ui, data, &search_term);
    }
}

/// Handles dropping entities on empty space (removes parents)
fn handle_empty_space_drop(ui: &mut egui::Ui, data: &mut NodeTreeTabData) {
    if data.drag_payload.is_some() && ui.input(|i| i.pointer.any_released()) {
        if data.drop_target.is_none() {
            data.drop_target = Some(Entity::PLACEHOLDER); 
        }
    }
}

/// Renders the tree in hierarchical mode (no search)
fn render_hierarchical_tree(ui: &mut egui::Ui, data: &mut NodeTreeTabData, search_term: &str) {
    let hierarchy_map = build_hierarchy_map(&data.hierarchy);

    if let Some(root_entities) = hierarchy_map.get(&None) {
        for (entity, name, entity_type) in root_entities {
            render_tree_node(
                ui,
                *entity,
                name,
                entity_type,
                &hierarchy_map,
                data,
                search_term,
            );
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

    for entry in &filtered {
        render_tree_node(
            ui,
            entry.entity,
            &entry.name,
            &entry.entity_type,
            &HashMap::new(), 
            data,
            search_term,
        );
    }

    ui.separator();
    ui.weak(format!("{} results found", filtered.len()));
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

/// Renders a single tree node with all its visual elements
fn render_tree_node(
    ui: &mut egui::Ui,
    entity: Entity,
    name: &str,
    entity_type: &str,
    hierarchy: &HashMap<Option<Entity>, Vec<(Entity, String, String)>>,
    data: &mut NodeTreeTabData,
    search_term: &str,
) {
    let hierarchy_entry = data.hierarchy.iter().find(|entry| entry.entity == entity);
    if hierarchy_entry.is_none() {
        return;
    }

    let entry = hierarchy_entry.unwrap();
    let has_children = hierarchy
        .get(&Some(entity))
        .map_or(false, |children| !children.is_empty());

    let visual_state = RowVisualState::from_hierarchy_entry(entry, data, has_children);

    // Calculate row rect for background and scrolling
    let available_rect = ui.available_rect_before_wrap();
    let row_height =
        ui.spacing().button_padding.y * 2.0 + ui.text_style_height(&egui::TextStyle::Button);
    let row_rect = egui::Rect::from_min_size(
        available_rect.min,
        egui::Vec2::new(available_rect.width(), row_height),
    );

    // Handle scrolling to selection
    if visual_state.is_active_selected && data.should_scroll_to_selection {
        ui.scroll_to_rect(row_rect, Some(egui::Align::Center));
        data.should_scroll_to_selection = false;
    }

    styling::draw_row_background(ui, &row_rect, &visual_state, search_term);

    let shift_held = ui.input(|i| i.modifiers.shift);
    let ctrl_held = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);

    ui.horizontal(|ui| {
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        let icon_size = ui.fonts(|f| f.row_height(&font_id));

        // Icon allocation for expand/collapse triangle
        let (icon_rect, icon_response) =
            ui.allocate_exact_size(egui::Vec2::new(icon_size, row_height), egui::Sense::click());

        ui.columns(2, |columns| {
            render_name_column(
                &mut columns[0],
                name,
                entity_type,
                &visual_state,
                search_term,
                data,
                entity,
                ctrl_held,
                shift_held,
            );

            render_type_column(
                &mut columns[1],
                entity,
                entity_type,
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
            search_term,
            icon_size,
        );

        // Handle expand/collapse clicks
        if has_children && icon_response.clicked() && search_term.is_empty() {
            if let Some(entry) = data.hierarchy.iter_mut().find(|e| e.entity == entity) {
                entry.is_expanded = !entry.is_expanded;
            }
        }

        if has_children && icon_response.hovered() && search_term.is_empty() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });

    render_children(ui, entity, hierarchy, data, &visual_state, search_term);
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

    // Create combined interaction area for click and drag
    let combined_response = ui.interact(
        button_response.rect,
        egui::Id::new(("tree_node", entity)),
        egui::Sense::click_and_drag(),
    );

    // Handle context menu (right-click)
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
        // Don't show anything for dummy parents (scene files)
        if visual_state.is_dummy_parent {
            return;
        }

        if verbose {
            // Show entity ID in non-curated mode
            ui.weak(format!("{}", entity.index()));
            ui.weak(":");
        }

        // Show entity type
        ui.label(entity_type);
    });
}

/// Renders child nodes if the parent is expanded
fn render_children(
    ui: &mut egui::Ui,
    entity: Entity,
    hierarchy: &HashMap<Option<Entity>, Vec<(Entity, String, String)>>,
    data: &mut NodeTreeTabData,
    visual_state: &RowVisualState,
    search_term: &str,
) {
    // Only show children when not searching and node is expanded
    if visual_state.has_children && visual_state.is_expanded && search_term.is_empty() {
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
                        search_term,
                    );
                }
            });
        }
    }
}

/// Styling functions for visual elements
pub mod styling {
    use super::*;
    use bevy_egui::egui;

    /// Draws the background for a tree row based on its visual state
    pub fn draw_row_background(
        ui: &mut egui::Ui,
        row_rect: &egui::Rect,
        visual_state: &RowVisualState,
        search_term: &str,
    ) {
        if visual_state.is_being_dragged {
            // Being dragged - use a tinted version of the selection color
            let drag_color = ui.style().visuals.selection.bg_fill.gamma_multiply(0.7);
            ui.painter().rect_filled(
                *row_rect,
                ui.style().visuals.menu_corner_radius / 2.,
                drag_color,
            );
        } else if visual_state.is_invalid_drop_target && search_term.is_empty() {
            // Invalid drop target - use error color
            let error_color = ui.style().visuals.error_fg_color.gamma_multiply(0.3);
            ui.painter().rect_filled(
                *row_rect,
                ui.style().visuals.menu_corner_radius / 2.,
                error_color,
            );
        } else if visual_state.is_valid_drop_target && search_term.is_empty() {
            // Valid drop target - could add highlighting here
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
