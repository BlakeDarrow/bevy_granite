use super::{
    despawn_rotate_gizmo, despawn_transform_gizmo, spawn_rotate_gizmo, spawn_transform_gizmo,
    DespawnGizmoEvent, GizmoType, LastSelectedGizmo, NewGizmoConfig, RotateGizmo,
    RotateGizmoParent, SpawnGizmoEvent, TransformGizmo, TransformGizmoParent,
};
use crate::{gizmos::NewGizmoType, selection::ActiveSelection};
use bevy::prelude::{
    Assets, Children, Commands, Entity, GlobalTransform, Mesh, MessageReader, MessageWriter, Query,
    Res, ResMut, StandardMaterial, With, Without,
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

pub fn gizmo_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut transform_query: Query<&GlobalTransform, Without<TransformGizmoParent>>,
    mut rotate_query: Query<&GlobalTransform, Without<RotateGizmoParent>>,
    selected_gizmo: Res<NewGizmoType>,
    mut spawn_events: MessageReader<SpawnGizmoEvent>,
    mut despawn_events: MessageReader<DespawnGizmoEvent>,
    mut transform_gizmo_query: Query<(Entity, &TransformGizmoParent, &Children)>,
    mut rotate_gizmo_query: Query<(Entity, &RotateGizmo, &Children)>,
    new_config: Res<NewGizmoConfig>,
) {
    for SpawnGizmoEvent(entity) in spawn_events.read() {
        if matches!(**selected_gizmo, GizmoType::Transform) {
            spawn_transform_gizmo(
                *entity,
                &mut transform_query,
                &mut commands,
                &mut meshes,
                &mut materials,
                new_config.transform(),
            );
        } else if matches!(**selected_gizmo, GizmoType::Rotate) {
            spawn_rotate_gizmo(
                *entity,
                &mut rotate_query,
                &mut commands,
                &mut materials,
                &mut meshes,
                new_config.rotation(),
            );
        }
    }

    for DespawnGizmoEvent(gizmo_type) in despawn_events.read() {
        if matches!(gizmo_type, GizmoType::Transform) {
            despawn_transform_gizmo(&mut commands, &mut transform_gizmo_query);
        } else if matches!(gizmo_type, GizmoType::Rotate) {
            despawn_rotate_gizmo(&mut commands, &mut rotate_gizmo_query);
        }
    }
}

pub fn gizmo_changed_watcher(
    selected_gizmo: Res<NewGizmoType>,
    mut last_selected_gizmo: ResMut<LastSelectedGizmo>,
    mut despawn_writer: MessageWriter<DespawnGizmoEvent>,
    mut spawn_writer: MessageWriter<SpawnGizmoEvent>,
    active_selection: Query<Entity, With<ActiveSelection>>,
) {
    if **selected_gizmo != last_selected_gizmo.value {
        //log!(
       //     LogType::Editor,
        //    LogLevel::OK,
        //    LogCategory::Entity,
        //    "Gizmo changed"
        //);
        despawn_writer.write(DespawnGizmoEvent(last_selected_gizmo.value));
        last_selected_gizmo.value = **selected_gizmo;

        if let Ok(active) = active_selection.single() {
            spawn_writer.write(SpawnGizmoEvent(active));
        }
    }
}
