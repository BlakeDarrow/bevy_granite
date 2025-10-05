use crate::{
    entities::editable::{RequestEntityUpdateFromClass, UserUpdatedPointLightEvent},
    PointLightData,
};
use bevy::{
    color::Color,
    ecs::{entity::Entity, message::MessageReader, system::Query},
    light::PointLight,
};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

impl PointLightData {
    /// Request an entity update with this data
    pub fn push_to_entity(
        &self,
        entity: Entity,
        request_update: &mut RequestEntityUpdateFromClass,
    ) {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Requesting point light entity update"
        );

        request_update
            .point_light
            .write(UserUpdatedPointLightEvent {
                entity,
                data: self.clone(),
            });
    }
}

pub fn update_point_light_system(
    mut reader: MessageReader<UserUpdatedPointLightEvent>,
    mut query: Query<(Entity, &mut PointLight)>,
) {
    for UserUpdatedPointLightEvent {
        entity: requested_entity,
        data: new,
    } in reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Heard point light update event: {}",
            requested_entity
        );
        if let Ok((_entity, mut point_light)) = query.get_mut(*requested_entity) {
            point_light.intensity = new.intensity;
            point_light.color = Color::linear_rgb(new.color.0, new.color.1, new.color.2);
            point_light.range = new.range;
            point_light.shadows_enabled = new.shadows_enabled;
        } else {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Entity,
                "Could not find point light on: {}",
                requested_entity
            );
        }
    }
}
