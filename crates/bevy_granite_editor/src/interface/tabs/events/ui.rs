use bevy::prelude::*;
use bevy_egui::egui;
use std::sync::Mutex;

pub struct EventInfo {
    pub struct_name: &'static str,
    pub event_names: &'static [&'static str],
    pub event_senders: Vec<Box<dyn Fn(&mut World) + Send + Sync>>,
}

pub struct EventRequest {
    pub struct_name: String,
    pub event_name: String,
}

lazy_static::lazy_static! {
    pub static ref EVENT_REQUEST_QUEUE: Mutex<Vec<EventRequest>> = Mutex::new(Vec::new());
}

lazy_static::lazy_static! {
    pub static ref EVENT_REGISTRY: Mutex<Vec<EventInfo>> = Mutex::new(Vec::new());
}

pub fn register_ui_callable_events_with_senders(
    struct_name: &'static str,
    event_names: &'static [&'static str],
    event_senders: Vec<Box<dyn Fn(&mut World) + Send + Sync>>,
) {
    EVENT_REGISTRY.lock().unwrap().push(EventInfo {
        struct_name,
        event_names,
        event_senders,
    });
}

#[derive(PartialEq, Clone, Default)]
pub struct EventsTabData {
    pub button_clicked: Option<String>,
}

pub fn events_tab_ui(ui: &mut egui::Ui, data: &mut EventsTabData) {
    let small_spacing = crate::UI_CONFIG.small_spacing;
    let spacing = crate::UI_CONFIG.spacing;
    let registry = EVENT_REGISTRY.lock().unwrap();
    if registry.is_empty() {
        ui.label("Events will appear here when structs with #[ui_callable_events] are processed.");
    } else {
        for event_info in registry.iter() {
            ui.group(|ui| {
                ui.label(format!("{}:", clean_name(event_info.struct_name)));
                ui.add_space(spacing);
                ui.set_width(ui.available_width());
                for event_name in event_info.event_names.iter() {
                    let clean_event_name = clean_name(event_name);
                    if ui.button(&clean_event_name).clicked() {
                        EVENT_REQUEST_QUEUE.lock().unwrap().push(EventRequest {
                            struct_name: event_info.struct_name.to_string(),
                            event_name: event_name.to_string(),
                        });
                        data.button_clicked = Some(clean_event_name);
                    }
                    ui.add_space(small_spacing);
                }
                ui.add_space(small_spacing);
            });
        }
    }
}

fn clean_name(name: &str) -> String {
    let mut result = String::new();
    let mut chars = name.chars().peekable();
    let mut is_first = true;

    while let Some(ch) = chars.next() {
        if ch == '_' {
            if !is_first {
                result.push(' ');
            }
        } else if ch.is_uppercase() && !is_first {
            result.push(' ');
            result.push(ch);
        } else if is_first {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
        } else {
            result.push(ch);
        }
        is_first = false;
    }

    result
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
