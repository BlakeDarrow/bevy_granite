use crate::{
    interface::{
        events::UserRequestGraniteTypeViaPopup, shared::widgets::make_frame_solid_via_context,
    },
    UI_CONFIG,
};
use bevy::{ecs::message::MessageWriter, prelude::Vec2};
use bevy_egui::{
    egui::{self, Window},
    EguiContexts,
};
use bevy_granite_core::{ClassCategory, GraniteType, GraniteTypes};

// We dont need EntityClassType. Its the same list as the data sister struct. just keep one struct - the data one

pub fn add_entity_ui(
    contexts: &mut EguiContexts,
    position: Vec2,
    mut entity_add_request: MessageWriter<UserRequestGraniteTypeViaPopup>,
) -> bool {
    let mut should_close = false;

    let spacing = UI_CONFIG.spacing;
    let small_spacing = UI_CONFIG.small_spacing;
    let _response = Window::new("Add Entity")
        .resizable(false)
        .title_bar(false)
        .fixed_pos([position.x, position.y])
        // call this to ensure the window is not transparent when theme transparency is selected
        .frame(make_frame_solid_via_context(
            egui::Frame::window(&contexts.ctx_mut().expect("Egui context to exist").style()),
            contexts.ctx_mut().expect("Egui context to exist"),
        ))
        .show(contexts.ctx_mut().expect("Egui context to exist"), |ui| {
            ui.horizontal(|ui| {
                let categories = get_all_categories();

                // Get or initialize the last hovered category from memory
                let popup_id = egui::Id::new("add_entity_last_category");
                let mut last_hovered_category = ui.memory(|mem| {
                    mem.data
                        .get_temp::<Option<(ClassCategory, Vec<GraniteTypes>)>>(popup_id)
                        .unwrap_or(None)
                });

                let mut current_hovered_category = None;

                // Left side - categories
                ui.vertical(|ui| {
                    ui.label("Add Entity:");
                    ui.add_space(spacing);

                    for category in categories {
                        let category_name = category.get_friendly_name();
                        let entities_in_category = GraniteTypes::all_by_category(category);

                        if entities_in_category.is_empty()
                            || (entities_in_category.len() == 1
                                && entities_in_category[0]
                                    == GraniteTypes::Unknown(Default::default()))
                        {
                            continue;
                        }

                        let button_response = ui.button(&category_name);
                        if button_response.hovered() {
                            current_hovered_category =
                                Some((category, entities_in_category.clone()));
                        }
                        ui.add_space(small_spacing);
                    }

                    ui.add_space(spacing);
                });

                // Update the stored category if we're hovering over a new one
                if let Some(new_category) = current_hovered_category {
                    last_hovered_category = Some(new_category);
                    ui.memory_mut(|mem| {
                        mem.data
                            .insert_temp(popup_id, last_hovered_category.clone());
                    });
                }

                // Right side - entity buttons for the stored category
                if let Some((_, entities)) = last_hovered_category {
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.add_space(spacing);

                        for entity_type in entities.iter() {
                            if let GraniteTypes::Unknown(_) = *entity_type {
                                continue;
                            }
                            if ui.button(entity_type.type_name()).clicked() {
                                entity_add_request.write(UserRequestGraniteTypeViaPopup {
                                    class: entity_type.clone(),
                                });
                                should_close = true;
                            }
                            ui.add_space(small_spacing);
                        }
                    });
                }
            });
        });

    let ctx = contexts.ctx_mut().expect("Egui context to exist");
    if ctx.input(|i| i.pointer.any_click()) && !ctx.is_pointer_over_area() {
        should_close = true;
    }
    should_close
}

fn get_all_categories() -> Vec<ClassCategory> {
    let mut categories = GraniteTypes::all()
        .into_iter()
        .map(|entity_type| entity_type.category())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    categories.sort();
    categories
}
