use bevy::{
    ecs::query::Changed,
    picking::hover::PickingInteraction,
    prelude::{Entity, Name, Query, Resource, Vec3},
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use crate::gizmos::GizmoMesh;

#[derive(Resource)]
pub struct RaycastCursorLast {
    pub position: Vec3,
}
#[derive(Resource)]
pub struct RaycastCursorPos {
    pub position: Vec3,
}

#[derive(PartialEq, Eq)]
pub enum HitType {
    Gizmo,
    Icon,
    Mesh,
    Vertex,
    Void,
    None,
}

pub fn raycast_at_cursor(
    query: Query<
        (Entity, Option<&GizmoMesh>, &Name, &PickingInteraction),
        Changed<PickingInteraction>,
    >,
) -> (Option<Entity>, HitType) {
    for (entity, gizmo, name, interaction) in query.iter() {
        if *interaction == PickingInteraction::Pressed {
            if gizmo.is_some() {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Input,
                    "Gizmo ray hit: {}",
                    name
                );
                return (Some(entity), HitType::Gizmo);
            } else {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Input,
                    "Mesh ray hit: {}",
                    name
                );
                return (Some(entity), HitType::Mesh);
            }
        }
    }
    (None, HitType::None)
}
