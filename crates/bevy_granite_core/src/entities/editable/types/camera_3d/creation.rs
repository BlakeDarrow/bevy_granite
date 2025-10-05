use super::Camera3D;
use crate::{
    entities::EntitySaveReadyData, GraniteEditorSerdeEntity, GraniteType, GraniteTypes,
    HasRuntimeData, IdentityData,
};
use bevy::{
    camera::{Camera, Camera3d},
    ecs::{bundle::Bundle, entity::Entity, system::Commands},
    prelude::Name,
    transform::components::Transform,
};
use uuid::Uuid;

impl Camera3D {
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
            class: GraniteTypes::Camera3D(self.clone()),
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

        if self.has_volumetric_fog {
            let mut fog = bevy::light::VolumetricFog::default();
            let mut fog_volume = bevy::light::FogVolume::default();

            if let Some(fog_settings) = &self.volumetric_fog_settings {
                fog.ambient_color = fog_settings.ambient_color;
                fog.ambient_intensity = fog_settings.ambient_intensity;
                fog.step_count = fog_settings.step_count;
                fog_volume.fog_color = fog_settings.fog_color;
                fog_volume.absorption = fog_settings.absorption;
                fog_volume.light_intensity = fog_settings.light_intensity;
                fog_volume.light_tint = fog_settings.light_tint;
                fog_volume.density_factor = fog_settings.density;
                fog_volume.scattering = fog_settings.scattering;
                fog_volume.scattering_asymmetry = fog_settings.scattering_asymmetry;

                // TODO: work out the bevy 0.16 equivalent for max_depth
                // entity.insert(VolumetricFogSettings {
                //     max_depth: fog_settings.max_depth,
                // });
            }
            //I don't know if the fog volume should be attached to the camera or its own entity
            entity.insert((fog, fog_volume));
        }
        entity.id()
    }

    /// Build a bundle that is ready to spawn from a Camera3D
    fn get_bundle(
        camera_3d: Camera3D,
        identity: IdentityData,
        transform: Transform,
    ) -> impl Bundle {
        (
            Camera3d::default(),
            Camera {
                is_active: camera_3d.is_active,
                ..Default::default()
            },
            transform,
            Name::new(identity.name.clone()),
            GraniteEditorSerdeEntity,
            HasRuntimeData,
            IdentityData {
                name: identity.name.clone(),
                uuid: identity.uuid.clone(),
                class: identity.class.clone(),
            },
        )
    }

    fn extract_class(identity: &IdentityData) -> Camera3D {
        match &identity.class {
            GraniteTypes::Camera3D(camera_data) => camera_data.clone(),
            _ => panic!("Expected Camera3D class data, got different type from save data"),
        }
    }
}
