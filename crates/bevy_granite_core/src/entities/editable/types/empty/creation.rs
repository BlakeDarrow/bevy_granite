use super::Empty;
use crate::{
    entities::EntitySaveReadyData, GraniteEditorSerdeEntity, GraniteType, GraniteTypes,
    HasRuntimeData, IdentityData,
};
use bevy::{
    camera::visibility::Visibility,
    ecs::{bundle::Bundle, entity::Entity, system::Commands},
    prelude::Name,
    transform::components::Transform,
};
use uuid::Uuid;

impl Empty {
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
            class: GraniteTypes::Empty(self.clone()),
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

    /// Build a bundle that is ready to spawn from an empty
    /// in this case, its a spatial bundle, so no class data needed
    fn get_bundle(_empty: Empty, identity: IdentityData, transform: Transform) -> impl Bundle {
        (
            Visibility::default(),
            transform,
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

    fn extract_class(identity: &IdentityData) -> Empty {
        match &identity.class {
            GraniteTypes::Empty(empty_data) => empty_data.clone(),
            _ => panic!("Expected Empty class data, got different type from save data"),
        }
    }
}
