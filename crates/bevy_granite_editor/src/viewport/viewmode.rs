use bevy::ecs::system::{Commands, Query, ResMut};
use bevy::light::DirectionalLight;
use bevy::prelude::{Entity, GlobalTransform, Name, Resource, Transform, With};
use bevy_granite_core::EditorIgnore;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use super::camera::EditorViewportCamera;

/// Scene light configuration - adds a directional light from the editor camera
#[derive(Resource)]
pub struct SceneLightState {
    /// Whether the scene light is enabled
    pub enabled: bool,
    /// Entity ID of the scene light (if spawned)
    pub light_entity: Option<Entity>,
}

impl Default for SceneLightState {
    fn default() -> Self {
        Self {
            enabled: false,
            light_entity: None,
        }
    }
}

impl SceneLightState {
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled != enabled {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::System,
                "Scene light: {}",
                if enabled { "enabled" } else { "disabled" }
            );
            self.enabled = enabled;
        }
    }
}

/// System that manages the scene light attached to the editor camera
/// Creates/removes a directional light that follows the camera's forward direction
pub fn scene_light_system(
    mut commands: Commands,
    mut scene_light_state: ResMut<SceneLightState>,
    camera_query: Query<&GlobalTransform, With<EditorViewportCamera>>,
) {
    let camera_transform = camera_query.single();

    if scene_light_state.enabled {
        // Scene light should be active
        if let Ok(cam_transform) = camera_transform {
            if let Some(light_entity) = scene_light_state.light_entity {
                // Update existing light to match camera direction
                if let Ok(mut entity_commands) = commands.get_entity(light_entity) {
                    entity_commands.insert(Transform::from_translation(cam_transform.translation())
                        .looking_to(cam_transform.forward(), cam_transform.up()));
                }
            } else {
                // Spawn new scene light
                let light_entity = commands
                    .spawn((
                        DirectionalLight {
                            shadows_enabled: false,
                            illuminance: 10000.0,
                            ..Default::default()
                        },
                        Transform::from_translation(cam_transform.translation())
                            .looking_to(cam_transform.forward(), cam_transform.up()),
                        Name::new("Scene Light"),
                        EditorIgnore::SERIALIZE | EditorIgnore::PICKING,
                    ))
                    .id();

                scene_light_state.light_entity = Some(light_entity);
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::System,
                    "Scene light spawned"
                );
            }
        }
    } else {
        // Scene light should be inactive - despawn if it exists
        if let Some(light_entity) = scene_light_state.light_entity.take() {
            commands.entity(light_entity).despawn();
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::System,
                "Scene light despawned"
            );
        }
    }
}

/// System that cleans up the scene light when editor becomes inactive
pub fn cleanup_scene_light_system(
    mut commands: Commands,
    mut scene_light_state: ResMut<SceneLightState>,
) {
    // Despawn scene light when editor deactivates
    if let Some(light_entity) = scene_light_state.light_entity.take() {
        commands.entity(light_entity).despawn();
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::System,
            "Scene light cleaned up (editor inactive)"
        );
    }
}
