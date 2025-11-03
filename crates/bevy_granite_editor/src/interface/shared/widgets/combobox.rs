use bevy_egui::egui::{self, Popup};
use bevy_granite_core::{AvailableEditableMaterials, EditableMaterial, ReflectedComponent};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use egui::{Align2, Rect, Response, Shape, Stroke, Ui, Vec2};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

// Generic trait for items that can be displayed in selectors
trait SelectableItem {
    fn display_name(&self) -> &str;
    fn search_text(&self) -> String;
    fn group_key(&self) -> String;
}

impl SelectableItem for String {
    fn display_name(&self) -> &str {
        if let Some(last_separator) = self.rfind("::") {
            &self[last_separator + 2..]
        } else {
            self
        }
    }

    fn search_text(&self) -> String {
        self.to_lowercase()
    }

    fn group_key(&self) -> String {
        if let Some(last_separator) = self.rfind("::") {
            self[..last_separator].to_string()
        } else {
            "Root".to_string()
        }
    }
}

impl SelectableItem for Cow<'static, str> {
    fn display_name(&self) -> &str {
        if let Some(last_separator) = self.rfind("::") {
            &self[last_separator + 2..]
        } else {
            self
        }
    }

    fn search_text(&self) -> String {
        self.to_lowercase()
    }

    fn group_key(&self) -> String {
        if let Some(last_separator) = self.rfind("::") {
            self[..last_separator].to_string()
        } else {
            "Root".to_string()
        }
    }
}

impl SelectableItem for EditableMaterial {
    fn display_name(&self) -> &str {
        &self.friendly_name
    }

    fn search_text(&self) -> String {
        format!(
            "{} {}",
            self.friendly_name.to_lowercase(),
            self.path.to_lowercase()
        )
    }

    fn group_key(&self) -> String {
        if self.path.is_empty() || self.path == "None" {
            "materials/internal".to_string()
        } else if let Some(last_separator) = self.path.rfind('/') {
            self.path[..last_separator].to_string()
        } else {
            "Root".to_string()
        }
    }
}

fn generic_selector_popup<T: SelectableItem>(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    button_response: &egui::Response,
    search_filter: &mut String,
    items: &[T],
    search_id_suffix: &str,
    no_items_message: &str,
    no_matches_message: &str,
    render_item: impl FnMut(&mut egui::Ui, &T) -> bool,
) -> bool {
    let mut popup_changed = false;

    if Popup::is_id_open(ui.ctx(), popup_id) {
        ui.memory_mut(|mem| mem.keep_popup_open(popup_id));
        let popup_pos = button_response.rect.left_bottom() + egui::vec2(0.0, 4.0);

        let area_response = egui::Area::new(popup_id)
            .fixed_pos(popup_pos)
            .order(egui::Order::TOP)
            .show(ui.ctx(), |ui: &mut egui::Ui| {
                let frame = super::make_frame_solid(egui::Frame::popup(ui.style()), ui);
                frame.show(ui, |ui: &mut egui::Ui| {
                    ui.set_min_width(button_response.rect.width());
                    ui.set_max_width(button_response.rect.width());
                    ui.set_max_height(400.0);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, true])
                        .show(ui, |ui: &mut egui::Ui| {
                            popup_changed = render_popup_content(
                                ui,
                                search_filter,
                                items,
                                search_id_suffix,
                                no_items_message,
                                no_matches_message,
                                render_item,
                            );
                        });
                })
            });

        // Close popup if clicked outside
        if ui.input(|i| i.pointer.any_click()) {
            let popup_rect = area_response.response.rect;
            let button_rect = button_response.rect;

            if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                if !popup_rect.contains(pointer_pos) && !button_rect.contains(pointer_pos) {
                    Popup::close_id(ui.ctx(), popup_id);
                }
            }
        }
    }

    popup_changed
}

fn render_popup_content<T: SelectableItem>(
    ui: &mut egui::Ui,
    search_filter: &mut String,
    items: &[T],
    search_id_suffix: &str,
    no_items_message: &str,
    no_matches_message: &str,
    mut render_item: impl FnMut(&mut egui::Ui, &T) -> bool,
) -> bool {
    let mut changed = false;

    // Search box
    render_search_box(ui, search_filter, search_id_suffix);

    if items.is_empty() {
        ui.label(no_items_message);
        return false;
    }

    // Filter items
    let filtered_items: Vec<_> = items
        .iter()
        .filter(|item| {
            search_filter.is_empty() || item.search_text().contains(&search_filter.to_lowercase())
        })
        .collect();

    if filtered_items.is_empty() {
        ui.label(no_matches_message);
        return false;
    }

    // Group and render items
    let mut grouped_items: HashMap<String, Vec<&T>> = HashMap::new();
    for item in filtered_items.iter() {
        grouped_items
            .entry(item.group_key())
            .or_default()
            .push(*item);
    }

    let mut sorted_groups: Vec<_> = grouped_items.into_iter().collect();
    sorted_groups.sort_by(|a, b| a.0.cmp(&b.0));

    let show_ungrouped =
        sorted_groups.len() == 1 && sorted_groups[0].0 == "Root" || !search_filter.is_empty();

    for (group_name, mut group_items) in sorted_groups {
        group_items.sort_by(|a, b| a.display_name().cmp(b.display_name()));

        if show_ungrouped {
            for item in group_items {
                if render_item(ui, item) {
                    changed = true;
                }
            }
        } else {
            let group_display_name = if group_name == "Root" {
                format!("Items ({})", group_items.len())
            } else {
                let clean_name = group_name.strip_prefix("materials/").unwrap_or(&group_name);
                format!("{} ({})", clean_name, group_items.len())
            };

            ui.collapsing(group_display_name, |ui| {
                for item in &group_items {
                    if render_item(ui, item) {
                        changed = true;
                    }
                }
            });
            ui.ctx().request_repaint();
        }
    }

    changed
}

fn render_search_box(ui: &mut egui::Ui, search_filter: &mut String, id_suffix: &str) {
    let spacing = crate::UI_CONFIG.spacing;
    let large_spacing = crate::UI_CONFIG.large_spacing;

    ui.add_space(spacing);
    ui.horizontal(|ui| {
        ui.add_space(spacing);
        ui.label("🔍");
        ui.add_space(large_spacing);

        let text_edit_id = egui::Id::new(format!("{}_search", id_suffix));
        let search_response = ui.add(
            egui::TextEdit::singleline(search_filter)
                .id(text_edit_id)
                .desired_width(ui.available_width() - large_spacing)
                .hint_text("Search..."),
        );

        if !search_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            ui.memory_mut(|mem| mem.request_focus(text_edit_id));
        }
    });

    ui.add_space(spacing);
    ui.separator();
    ui.add_space(spacing);
}

pub fn paint_dropdown_arrow(ui: &Ui, rect: Rect, visuals: &egui::style::WidgetVisuals) {
    let arrow_rect = Rect::from_center_size(
        rect.center(),
        Vec2::new(rect.width() * 0.7, rect.height() * 0.45),
    );

    ui.painter().add(Shape::convex_polygon(
        vec![
            arrow_rect.left_top(),
            arrow_rect.right_top(),
            arrow_rect.center_bottom(),
        ],
        visuals.fg_stroke.color,
        Stroke::NONE,
    ));
}

fn combobox_style_button(ui: &mut Ui, button_text: &str) -> Response {
    let icon_spacing = ui.spacing().icon_spacing;
    let icon_size = Vec2::splat(ui.spacing().icon_width);
    let margin = ui.spacing().button_padding;

    let total_width = ui.available_width();
    let total_height = ui.spacing().interact_size.y + 16.0;

    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(total_width, total_height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let text_color = if response.hovered() || response.is_pointer_button_down_on() {
            ui.visuals()
                .override_text_color
                .unwrap_or_else(|| ui.visuals().strong_text_color())
        } else {
            ui.visuals()
                .override_text_color
                .unwrap_or_else(|| ui.visuals().text_color())
        };

        let text_galley = ui.fonts(|f| {
            f.layout_no_wrap(
                button_text.to_string(),
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            )
        });

        ui.painter().rect(
            rect.expand(visuals.expansion),
            visuals.corner_radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Middle,
        );

        let inner_rect = rect.shrink2(margin);
        let icon_rect = Align2::RIGHT_CENTER.align_size_within_rect(icon_size, inner_rect);
        let text_width = inner_rect.width() - icon_spacing - icon_size.x;
        let text_rect =
            Rect::from_min_size(inner_rect.min, Vec2::new(text_width, inner_rect.height()));

        ui.painter().galley(
            Align2::LEFT_CENTER
                .align_size_within_rect(text_galley.size(), text_rect)
                .min,
            text_galley,
            text_color,
        );

        paint_dropdown_arrow(ui, icon_rect, visuals);
    }

    response
}

fn handle_popup_button(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    button_text: &str,
    search_filter: &mut String,
) -> egui::Response {
    let button_response = combobox_style_button(ui, button_text);
    if button_response.clicked() {
        let is_open = Popup::is_id_open(ui.ctx(), popup_id);
        if is_open {
            Popup::close_id(ui.ctx(), popup_id);
        } else {
            Popup::open_id(ui.ctx(), popup_id);
            search_filter.clear();
        }
    }

    button_response
}

pub fn component_selector_combo(
    ui: &mut egui::Ui,
    search_filter: &mut String,
    registered_type_names: Vec<Cow<'static, str>>,
    existing_components: &[ReflectedComponent],
    component_changed: &mut bool,
    registered_add_request: &mut Option<String>,
) -> bool {
    let popup_id = egui::Id::new("component_selector_popup");

    let existing_type_names: HashSet<Cow<'static, str>> = existing_components
        .iter()
        .map(|comp| comp.type_name.clone())
        .collect();

    let available_components: Vec<_> = registered_type_names
        .iter()
        .filter(|name| !existing_type_names.contains(name.as_ref()))
        .cloned()
        .collect();

    let dropdown_text = if available_components.is_empty() {
        "No components available"
    } else {
        "Components..."
    };

    let button_response = handle_popup_button(ui, popup_id, dropdown_text, search_filter);

    generic_selector_popup(
        ui,
        popup_id,
        &button_response,
        search_filter,
        &available_components,
        "component",
        "All registered components are already on this entity",
        "No components match your search",
        |ui, component_name: &Cow<'static, str>| {
            if ui
                .selectable_label(false, component_name.display_name())
                .clicked()
            {
                *component_changed = true;
                *registered_add_request = Some(component_name.to_string());
                Popup::close_id(ui.ctx(), popup_id);
                return true;
            }
            false
        },
    )
}

pub fn material_selector_combo(
    ui: &mut egui::Ui,
    search_filter: &mut String,
    available_materials: &AvailableEditableMaterials,
    class_materal_path: &mut String,
    current_material: &mut EditableMaterial,
) -> bool {
    let popup_id = egui::Id::new("material_selector_popup");

    let dropdown_text = if available_materials.materials.is_some() {
        if current_material.is_empty() {
            "None"
        } else {
            current_material.path.as_str()
        }
    } else {
        "No materials available"
    };

    let button_response = handle_popup_button(ui, popup_id, dropdown_text, search_filter);

    if let Some(ref obj_materials) = available_materials.materials {
        generic_selector_popup(
            ui,
            popup_id,
            &button_response,
            search_filter,
            obj_materials,
            "material",
            "None",
            "No materials match your search",
            |ui, new_material| {
                let is_selected = *current_material.friendly_name == new_material.friendly_name
                    && *current_material.path == new_material.path;

                if ui
                    .selectable_label(is_selected, &new_material.friendly_name)
                    .clicked()
                {
                    current_material.friendly_name = new_material.friendly_name.clone();
                    current_material.path = new_material.path.clone();

                    *current_material = new_material.clone();
                    *class_materal_path = new_material.path.clone();

                    log!(
                        LogType::Editor,
                        LogLevel::OK,
                        LogCategory::UI,
                        "User selected new material: {}",
                        new_material.friendly_name
                    );

                    Popup::close_id(ui.ctx(), popup_id);
                    return true;
                }
                false
            },
        )
    } else {
        false
    }
}
