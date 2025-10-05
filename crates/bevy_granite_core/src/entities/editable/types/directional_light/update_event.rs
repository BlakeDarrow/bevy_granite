use crate::{
    entities::editable::{RequestEntityUpdateFromClass, UserUpdatedDirectionalLightEvent},
    DirLight,
};
use bevy::{
    color::Color,
    ecs::{
        entity::Entity,
        message::MessageReader,
        system::{Commands, Query},
    },
    light::{DirectionalLight, VolumetricLight},
};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

impl DirLight {
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
            "Requesting directional light entity update"
        );

        request_update
            .directional_light
            .write(UserUpdatedDirectionalLightEvent {
                entity,
                data: self.clone(),
            });
    }
}

/// Actually update the specific entity with the class data
/// In the future im sure we will have FOV and what not
pub fn update_directional_light_system(
    mut reader: MessageReader<UserUpdatedDirectionalLightEvent>,
    mut query: Query<(Entity, &mut DirectionalLight)>,
    mut commands: Commands,
) {
    for UserUpdatedDirectionalLightEvent {
        entity: requested_entity,
        data: new,
    } in reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Heard directional light update event: {}",
            requested_entity
        );

        if let Ok((entity, mut directional_light)) = query.get_mut(*requested_entity) {
            directional_light.illuminance = new.illuminance;
            directional_light.color = Color::linear_rgb(new.color.0, new.color.1, new.color.2);
            directional_light.shadows_enabled = new.shadows_enabled;
            if new.volumetric {
                commands.entity(entity).insert(VolumetricLight);
            } else {
                commands.entity(entity).remove::<VolumetricLight>();
            }
        } else {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Entity,
                "Could not find directional light on: {}",
                requested_entity
            );
        }
    }
}
