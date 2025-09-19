use bevy_egui::egui;
use bevy::prelude::*;
use std::sync::Mutex;

// Registry for UI callable events - stores type information and event senders
pub struct EventInfo {
    pub struct_name: &'static str,
    pub event_names: &'static [&'static str],
    pub event_senders: Vec<Box<dyn Fn(&mut World) + Send + Sync>>,
}

// Queue for pending event requests
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
    let spacing = crate::UI_CONFIG.spacing;
    
    ui.label("UI Callable Events");
    ui.add_space(spacing);
    
    ui.separator(); 
    
    // Dynamically create buttons from registry
    let registry = EVENT_REGISTRY.lock().unwrap();
    if registry.is_empty() {
        ui.label("No UI callable events registered yet.");
        ui.add_space(spacing);
        ui.label("Events will appear here when structs with #[ui_callable_events] are processed.");
    } else {
        for event_info in registry.iter() {
            ui.label(format!("{}:", event_info.struct_name));
            ui.add_space(spacing * 0.5);
            
            for event_name in event_info.event_names.iter() {
                if ui.button(*event_name).clicked() {
                    // Queue the event request
                    EVENT_REQUEST_QUEUE.lock().unwrap().push(EventRequest {
                        struct_name: event_info.struct_name.to_string(),
                        event_name: event_name.to_string(),
                    });
                    data.button_clicked = Some(event_name.to_string());
                }
            }
            ui.add_space(spacing);
        }
    }
}