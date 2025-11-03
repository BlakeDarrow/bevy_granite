use crate::entities::editable::{RequestEntityUpdateFromClass, UserUpdatedSpotLightEvent};
use super::SpotLightData;
use bevy::{
    color::Color,
    ecs::{entity::Entity, message::MessageReader, system::Query},
    light::SpotLight,
};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

impl SpotLightData {
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
            "Requesting spot light entity update"
        );

        request_update
            .spot_light
            .write(UserUpdatedSpotLightEvent {
                entity,
                data: self.clone(),
            });
    }
}

pub fn update_spot_light_system(
    mut reader: MessageReader<UserUpdatedSpotLightEvent>,
    mut query: Query<(Entity, &mut SpotLight)>,
) {
    for UserUpdatedSpotLightEvent {
        entity: requested_entity,
        data: new,
    } in reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Heard spot light update event: {}",
            requested_entity
        );
        if let Ok((_entity, mut spot_light)) = query.get_mut(*requested_entity) {
            spot_light.color = Color::linear_rgb(new.color.0, new.color.1, new.color.2);
            spot_light.intensity = new.intensity;
            spot_light.range = new.range;
            spot_light.radius = new.radius;
            spot_light.shadows_enabled = new.shadows_enabled;
            spot_light.shadow_depth_bias = new.shadow_depth_bias;
            spot_light.shadow_normal_bias = new.shadow_normal_bias;
            spot_light.inner_angle = new.inner_angle;
            spot_light.outer_angle = new.outer_angle;
        } else {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Entity,
                "Could not find spot light on: {}",
                requested_entity
            );
        }
    }
}
