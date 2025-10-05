use crate::entities::editable::RequestEntityUpdateFromClass;
use crate::entities::editable::UserUpdatedEmptyEvent;
use crate::entities::Empty;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageReader;
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
/// In the future im sure we will have FOV and what not
pub fn update_empty_system(mut reader: MessageReader<UserUpdatedEmptyEvent>) {
    for UserUpdatedEmptyEvent {
        entity: requested_entity,
        data: _new,
    } in reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Heard empty update event: {}",
            requested_entity
        );
        // Nothing to do here yet
    }
}
