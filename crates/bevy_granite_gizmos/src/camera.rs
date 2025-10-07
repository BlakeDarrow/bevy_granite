use bevy::{
    camera::{visibility::RenderLayers, Camera, Camera3d},
    ecs::{
        component::Component,
        entity::Entity,
        message::{Message, MessageReader, MessageWriter},
        query::{Added, With, Without},
        system::{Commands, Query},
    },
    prelude::Name,
    transform::components::Transform,
};
use bevy_granite_core::{MainCamera, TreeHiddenEntity, UICamera};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

#[derive(Component)]
pub struct GizmoCamera;

#[derive(Message)]
pub struct MainCameraAdded;

pub fn watch_for_main_camera_addition(
    main_camera_added: Query<Entity, Added<MainCamera>>,
    mut event_writer: MessageWriter<MainCameraAdded>,
) {
    if !main_camera_added.is_empty() {
        event_writer.write(MainCameraAdded);
    }
}

// Add Gizmo camera
pub fn add_gizmo_camera(
    gizmo_camera_query: Query<&mut Transform, With<GizmoCamera>>,
    main_camera_query: Query<
        &mut Transform,
        (With<MainCamera>, Without<GizmoCamera>, Without<UICamera>),
    >,
    mut main_camera_added: MessageReader<MainCameraAdded>,
    mut commands: Commands,
) {
    for _event in main_camera_added.read() {
        if !gizmo_camera_query.is_empty() {
            // No need to create, likely spawned via editor instead
            // If this plugin is being used without the editor, we will have an independent gizmo camera
            // If being used alongside the editor, no need to create a gizmo cam as we have UI cam
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Entity,
                "GizmoCamera already exists, skipping spawn",
            );

            return;
        }
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "MainCamera was added to an entity and it did not include GizmoCamera, so we will create one",
        );

        let Ok(main_camera_transform) = main_camera_query.single() else {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::Entity,
                "Could not find MainCamera to spawn GizmoCamera at",
            );
            return;
        };
        let _ui_camera = commands
            .spawn((
                *main_camera_transform,
                Camera3d::default(),
                Name::new("Gizmo Camera"),
            ))
            .insert(Camera {
                order: 1,
                clear_color: bevy::camera::ClearColorConfig::None,
                ..Default::default()
            })
            .insert(TreeHiddenEntity)
            .insert(GizmoCamera)
            .insert(RenderLayers::from_layers(&[14])) // 14 is our Gizmo layer.
            .id();
    }
}
