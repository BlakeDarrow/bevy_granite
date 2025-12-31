use crate::entities::editable::RequestEntityUpdateFromClass;
use crate::entities::editable::UserUpdatedEmptyEvent;
use crate::entities::Empty;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageReader;
use bevy::ecs::system::Commands;
use bevy::prelude::{Children, Query, Visibility};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

impl Empty {
    pub fn push_to_entity(
        &self,
        entity: Entity,
        request_update: &mut RequestEntityUpdateFromClass,
    ) {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Requesting directional light entity update"
        );

        request_update.empty.write(UserUpdatedEmptyEvent {
            entity,
            data: self.clone(),
        });
    }
}

/// Actually update the specific entity with the class data
/// Handles hiding/showing children based on hide_children flag
pub fn update_empty_system(
    mut reader: MessageReader<UserUpdatedEmptyEvent>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    for UserUpdatedEmptyEvent {
        entity: requested_entity,
        data: new,
    } in reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Heard empty update event: {} with hide_children={}",
            requested_entity,
            new.hide_children
        );
        
        // Update visibility of all children
        if let Ok(children) = children_query.get(*requested_entity) {
            let visibility = if new.hide_children {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            
            for &child in children.iter() {
                commands.entity(child).insert(visibility);
            }
        }
    }
}
