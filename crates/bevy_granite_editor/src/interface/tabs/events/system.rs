use bevy::prelude::*;
use super::ui::{EVENT_REQUEST_QUEUE, EVENT_REGISTRY};

pub fn send_queued_events_system(world: &mut World) {
    let mut queue = EVENT_REQUEST_QUEUE.lock().unwrap();
    
    if !queue.is_empty() {
        let registry = EVENT_REGISTRY.lock().unwrap();
        
        for request in queue.drain(..) {
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
