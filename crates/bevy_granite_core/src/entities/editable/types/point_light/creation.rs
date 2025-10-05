use super::PointLightData;
use crate::{
    entities::EntitySaveReadyData, GraniteEditorSerdeEntity, GraniteType, GraniteTypes,
    HasRuntimeData, IdentityData,
};
use bevy::{
    color::Color,
    ecs::{bundle::Bundle, entity::Entity, system::Commands},
    light::PointLight,
    prelude::Name,
    transform::components::Transform,
};
use uuid::Uuid;

impl PointLightData {
    /// Extract needed info to spawn this entity via save data
    pub fn spawn_from_save_data(
        save_data: &EntitySaveReadyData,
        commands: &mut Commands,
    ) -> Entity {
        let identity = &save_data.identity;
        let save_transform = &save_data.transform;

        Self::spawn_from_identity(commands, identity, save_transform.to_bevy())
    }

    /// Take the name and class from identity to spawn
    pub fn spawn_from_identity(
        commands: &mut Commands,
        identity: &IdentityData,
        transform: Transform,
    ) -> Entity {
        let class = Self::extract_class(&identity);

        class.spawn(identity, commands, transform)
    }

    /// Generally to be used from UI popups as it gives default name
    pub fn spawn_from_new_identity(&self, commands: &mut Commands, transform: Transform) -> Entity {
        let identity = IdentityData {
            name: self.type_name(),
            uuid: Uuid::new_v4(),
            class: GraniteTypes::PointLightData(self.clone()),
        };
        self.spawn(&identity, commands, transform)
    }

    /// Private core logic
    fn spawn(
        &self,
        identity: &IdentityData,
        commands: &mut Commands,
        transform: Transform,
    ) -> Entity {
        commands
            .spawn(Self::get_bundle(self.clone(), identity.clone(), transform))
            .id()
    }

    /// Build a bundle that is ready to spawn from a Point Light
    fn get_bundle(
        point_light: PointLightData,
        identity: IdentityData,
        transform: Transform,
    ) -> impl Bundle {
        (
            transform,
            PointLight {
                intensity: point_light.intensity,
                color: Color::linear_rgb(
                    point_light.color.0,
                    point_light.color.1,
                    point_light.color.2,
                ),
                range: point_light.range,
                shadows_enabled: point_light.shadows_enabled,
                ..Default::default()
            },
            Name::new(identity.name.clone()),
            HasRuntimeData,
            GraniteEditorSerdeEntity,
            IdentityData {
                name: identity.name.clone(),
                uuid: identity.uuid.clone(),
                class: identity.class.clone(),
            },
        )
    }

    fn extract_class(identity: &IdentityData) -> PointLightData {
        match &identity.class {
            GraniteTypes::PointLightData(point_light_data) => point_light_data.clone(),
            _ => panic!("Expected PointLightData class data, got different type from save data"),
        }
    }
}
