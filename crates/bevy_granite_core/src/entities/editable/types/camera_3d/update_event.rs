use super::{UserUpdatedCamera3DEvent, VolumetricFog};
use crate::{
    entities::editable::RequestEntityUpdateFromClass, Camera3D, GraniteTypes, IdentityData,
};
use bevy::{
    camera::Camera,
    ecs::{
        entity::Entity,
        message::MessageReader,
        system::{Commands, Query},
    },
    light::{FogVolume, VolumetricFog as VolumetricFogSettings},
};

use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

impl Camera3D {
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
            "Requesting camera entity update"
        );

        request_update.camera_3d.write(UserUpdatedCamera3DEvent {
            entity,
            data: self.clone(),
        });
    }
}

/// Actually update the specific entity with the class data
/// In the future im sure we will have FOV and what not
pub fn update_camera_3d_system(
    mut reader: MessageReader<UserUpdatedCamera3DEvent>,
    mut query: Query<(Entity, &mut Camera, &mut IdentityData)>,
    mut commands: Commands,
) {
    for UserUpdatedCamera3DEvent {
        entity: requested_entity,
        data: new,
    } in reader.read()
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Heard camera3d update event: {}",
            requested_entity
        );
        if let Ok((entity, mut camera, mut identity_data)) = query.get_mut(*requested_entity) {
            if new.is_active {
                camera.is_active = true;
            } else {
                camera.is_active = false;
            }

            if new.has_volumetric_fog {
                let fog_config = new.volumetric_fog_settings.clone().unwrap_or_default();
                let mut fog = VolumetricFogSettings::default();
                let mut fog_volume = FogVolume::default();
                fog.ambient_color = fog_config.ambient_color;
                fog.ambient_intensity = fog_config.ambient_intensity;
                fog_volume.fog_color = fog_config.fog_color;
                fog_volume.absorption = fog_config.absorption;
                fog.step_count = fog_config.step_count;
                fog_volume.light_intensity = fog_config.light_intensity;
                fog_volume.light_tint = fog_config.light_tint;
                fog_volume.density_factor = fog_config.density;
                fog_volume.scattering = fog_config.scattering;
                fog_volume.scattering_asymmetry = fog_config.scattering_asymmetry;

                //TODO: work out the bevy 0.16 equivalent for max_depth
                // commands.entity(entity).insert(VolumetricFogSettings {
                //     max_depth: new_fog.max_depth,
                // });
                commands.entity(entity).insert((fog, fog_volume));
            } else {
                commands
                    .entity(entity)
                    .remove::<(VolumetricFogSettings, FogVolume)>();
            }

            // Update the IdentityData to match new changes
            if let GraniteTypes::Camera3D(ref mut camera_data) = identity_data.class {
                camera_data.is_active = new.is_active;
                camera_data.has_volumetric_fog = new.has_volumetric_fog;

                if new.has_volumetric_fog {
                    // Ensure volumetric_fog_settings is populated
                    if camera_data.volumetric_fog_settings.is_none() {
                        camera_data.volumetric_fog_settings = Some(VolumetricFog::default());
                    }
                } else {
                    camera_data.volumetric_fog_settings = None;
                }
            }
        } else {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Entity,
                "Could not find camera on: {}",
                requested_entity
            );
        }
    }
}
