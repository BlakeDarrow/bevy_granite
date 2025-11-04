use super::{AtmosphereRenderingMethod, UserUpdatedCamera3DEvent};
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
    math::{UVec2, UVec3},
    pbr::{Atmosphere, AtmosphereMode, AtmosphereSettings as BevyAtmosphereSettings},
    post_process::bloom::{Bloom, BloomCompositeMode as BevyBloomCompositeMode},
    render::view::Hdr,
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
            "Heard camera3d update event: {} - has_atmosphere: {}",
            requested_entity,
            new.has_atmosphere
        );
        if let Ok((entity, mut camera, mut identity_data)) = query.get_mut(*requested_entity) {
            if new.is_active {
                camera.is_active = true;
            } else {
                camera.is_active = false;
            }
            
            // Update camera render order
            camera.order = new.order;

            // Handle dithering
            if new.dither {
                commands.entity(entity).insert(bevy::core_pipeline::tonemapping::DebandDither::Enabled);
            } else {
                commands.entity(entity).remove::<bevy::core_pipeline::tonemapping::DebandDither>();
            }

            // Handle bloom - requires HDR
            if new.has_bloom {
                // Bloom requires HDR to be enabled
                commands.entity(entity).insert(Hdr);
                
                let bloom_config = new.bloom_settings.clone().unwrap_or_default();
                let bloom = Bloom {
                    intensity: bloom_config.intensity,
                    low_frequency_boost: bloom_config.low_frequency_boost,
                    low_frequency_boost_curvature: bloom_config.low_frequency_boost_curvature,
                    high_pass_frequency: bloom_config.high_pass_frequency,
                    composite_mode: match bloom_config.composite_mode {
                        super::BloomCompositeMode::EnergyConserving => BevyBloomCompositeMode::EnergyConserving,
                        super::BloomCompositeMode::Additive => BevyBloomCompositeMode::Additive,
                    },
                    ..Default::default()
                };
                commands.entity(entity).insert(bloom);
            } else {
                commands.entity(entity).remove::<Bloom>();
                // Note: We don't remove HDR here as it might be needed for atmosphere
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

            // Handle atmosphere settings
            if new.has_atmosphere {
                let atmos_config = new.atmosphere_settings.clone().unwrap_or_default();

                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Applying atmosphere - bottom_radius: {}, top_radius: {}",
                    atmos_config.bottom_radius,
                    atmos_config.top_radius
                );

                // The UI can populate these with EARTH preset values if the checkbox is enabled
                let atmosphere = Atmosphere {
                    bottom_radius: atmos_config.bottom_radius,
                    top_radius: atmos_config.top_radius,
                    ground_albedo: atmos_config.ground_albedo.into(),
                    rayleigh_density_exp_scale: atmos_config.rayleigh_density_exp_scale,
                    rayleigh_scattering: atmos_config.rayleigh_scattering.into(),
                    mie_density_exp_scale: atmos_config.mie_density_exp_scale,
                    mie_scattering: atmos_config.mie_scattering,
                    mie_absorption: atmos_config.mie_absorption,
                    mie_asymmetry: atmos_config.mie_asymmetry,
                    ozone_layer_altitude: atmos_config.ozone_layer_altitude,
                    ozone_layer_width: atmos_config.ozone_layer_width,
                    ozone_absorption: atmos_config.ozone_absorption.into(),
                };

                commands.entity(entity).insert(Hdr);
                commands.entity(entity).insert(atmosphere);

                // Convert rendering method
                let rendering_mode = match atmos_config.rendering_method {
                    AtmosphereRenderingMethod::LookupTexture => AtmosphereMode::LookupTexture,
                    AtmosphereRenderingMethod::Raymarched => AtmosphereMode::Raymarched,
                };

                commands.entity(entity).insert(BevyAtmosphereSettings {
                    transmittance_lut_size: UVec2::new(
                        atmos_config.transmittance_lut_size.0,
                        atmos_config.transmittance_lut_size.1,
                    ),
                    multiscattering_lut_size: UVec2::new(
                        atmos_config.multiscattering_lut_size.0,
                        atmos_config.multiscattering_lut_size.1,
                    ),
                    sky_view_lut_size: UVec2::new(
                        atmos_config.sky_view_lut_size.0,
                        atmos_config.sky_view_lut_size.1,
                    ),
                    aerial_view_lut_size: UVec3::new(
                        atmos_config.aerial_view_lut_size.0,
                        atmos_config.aerial_view_lut_size.1,
                        atmos_config.aerial_view_lut_size.2,
                    ),
                    transmittance_lut_samples: atmos_config.transmittance_lut_samples,
                    multiscattering_lut_dirs: atmos_config.multiscattering_lut_dirs,
                    multiscattering_lut_samples: atmos_config.multiscattering_lut_samples,
                    sky_view_lut_samples: atmos_config.sky_view_lut_samples,
                    aerial_view_lut_samples: atmos_config.aerial_view_lut_samples,
                    aerial_view_lut_max_distance: atmos_config.aerial_view_lut_max_distance,
                    scene_units_to_m: atmos_config.scene_units_to_m,
                    sky_max_samples: atmos_config.sky_max_samples,
                    rendering_method: rendering_mode,
                    ..Default::default()
                });
            } else {
                commands
                    .entity(entity)
                    .remove::<(Atmosphere, BevyAtmosphereSettings, Hdr)>();
                
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Removed atmosphere from camera: {}",
                    entity
                );
            }

            if let GraniteTypes::Camera3D(ref mut camera_data) = identity_data.class {
                camera_data.is_active = new.is_active;
                camera_data.order = new.order;
                camera_data.dither = new.dither;
                camera_data.has_bloom = new.has_bloom;
                camera_data.has_volumetric_fog = new.has_volumetric_fog;
                camera_data.has_atmosphere = new.has_atmosphere;

                if new.has_bloom {
                    camera_data.bloom_settings = new.bloom_settings.clone();
                } else {
                    camera_data.bloom_settings = None;
                }

                if new.has_volumetric_fog {
                    camera_data.volumetric_fog_settings = new.volumetric_fog_settings.clone();
                } else {
                    camera_data.volumetric_fog_settings = None;
                }

                if new.has_atmosphere {
                    camera_data.atmosphere_settings = new.atmosphere_settings.clone();
                } else {
                    camera_data.atmosphere_settings = None;
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
