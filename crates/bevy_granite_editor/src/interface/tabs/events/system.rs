use crate::interface::BottomDockState;
use bevy::prelude::*;
use super::ui::{EVENT_REQUEST_QUEUE, EVENT_REGISTRY};

pub fn update_events_tab_system(
    mut bottom_dock_state: ResMut<BottomDockState>,
) {
    let tabs = &mut bottom_dock_state.dock_state.iter_all_tabs_mut();

    for (_, tab) in tabs {
        if let crate::interface::panels::BottomTab::Events { data: _data } = tab {
            // Process any queued event requests
            let mut queue = EVENT_REQUEST_QUEUE.lock().unwrap();
            
            if !queue.is_empty() {
                println!("Processing {} event requests", queue.len());
                
                // For now, just log the events - actual sending will be handled by a separate exclusive system
                for request in queue.drain(..) {
                    println!("Event queued: {} from {}", request.event_name, request.struct_name);
                }
            }
        }
    }
}

// Separate exclusive system to actually send the events
pub fn send_queued_events_system(world: &mut World) {
    let mut queue = EVENT_REQUEST_QUEUE.lock().unwrap();
    
    if !queue.is_empty() {
        let registry = EVENT_REGISTRY.lock().unwrap();
        
        // Process each event request
        for request in queue.drain(..) {
            // Find the matching event info and sender
            for event_info in registry.iter() {
                if event_info.struct_name == request.struct_name {
                    if let Some(index) = event_info.event_names.iter().position(|&name| name == request.event_name) {
                        if let Some(sender) = event_info.event_senders.get(index) {
                            sender(world);
                            println!("Successfully sent event: {}", request.event_name);
                            break;
                        }
                    }
                }
            }
        }
    }
}
