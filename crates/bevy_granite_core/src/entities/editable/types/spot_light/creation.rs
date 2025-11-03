use super::SpotLightData;
use crate::{
    entities::EntitySaveReadyData, GraniteEditorSerdeEntity, GraniteType, GraniteTypes,
    HasRuntimeData, IdentityData,
};
use bevy::{
    color::Color,
    ecs::{bundle::Bundle, entity::Entity, system::Commands},
    light::SpotLight,
    prelude::Name,
    transform::components::Transform,
};
use uuid::Uuid;

impl SpotLightData {
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
            class: GraniteTypes::SpotLightData(self.clone()),
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

    /// Build a bundle that is ready to spawn from a Spot Light
    fn get_bundle(
        spot_light: SpotLightData,
        identity: IdentityData,
        transform: Transform,
    ) -> impl Bundle {
        (
            transform,
            SpotLight {
                color: Color::linear_rgb(
                    spot_light.color.0,
                    spot_light.color.1,
                    spot_light.color.2,
                ),
                intensity: spot_light.intensity,
                range: spot_light.range,
                radius: spot_light.radius,
                shadows_enabled: spot_light.shadows_enabled,
                shadow_depth_bias: spot_light.shadow_depth_bias,
                shadow_normal_bias: spot_light.shadow_normal_bias,
                inner_angle: spot_light.inner_angle,
                outer_angle: spot_light.outer_angle,
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

    fn extract_class(identity: &IdentityData) -> SpotLightData {
        match &identity.class {
            GraniteTypes::SpotLightData(spot_light_data) => spot_light_data.clone(),
            _ => panic!("Expected SpotLightData class data, got different type from save data"),
        }
    }
}
