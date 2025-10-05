use super::DirLight;
use crate::{
    entities::EntitySaveReadyData, GraniteEditorSerdeEntity, GraniteType, GraniteTypes,
    HasRuntimeData, IdentityData,
};
use bevy::{
    color::Color,
    ecs::{bundle::Bundle, entity::Entity, system::Commands},
    light::{DirectionalLight, VolumetricLight},
    prelude::Name,
    transform::components::Transform,
};
use uuid::Uuid;

impl DirLight {
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
            class: GraniteTypes::DirLight(self.clone()),
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
        let mut entity =
            commands.spawn(Self::get_bundle(self.clone(), identity.clone(), transform));

        if self.volumetric {
            entity.insert(VolumetricLight);
        }
        entity.id()
    }

    /// Build the basic bundle
    fn get_bundle(
        dir_light: DirLight,
        identity: IdentityData,
        transform: Transform,
    ) -> impl Bundle {
        (
            transform,
            DirectionalLight {
                color: Color::linear_rgb(dir_light.color.0, dir_light.color.1, dir_light.color.2),
                illuminance: dir_light.illuminance,
                shadows_enabled: dir_light.shadows_enabled,
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

    fn extract_class(identity: &IdentityData) -> DirLight {
        match &identity.class {
            GraniteTypes::DirLight(dir_data) => dir_data.clone(),
            _ => panic!("Expected Directional Light class data, got different type from save data"),
        }
    }
}
