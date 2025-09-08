use bevy::{
    ecs::query::Changed,
    picking::hover::PickingInteraction,
    prelude::{Entity, Name, Query, Resource, Vec3},
};
use bevy_granite_core::IconProxy;
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
    Void,
    None,
}

// pub fn raycast_at_cursor(
//     cursor_ray: &Res<CursorRay>,
//     raycast: &mut Raycast,
//     query: Query<(Entity, Option<&GizmoMesh>, Option<&IconProxy>, &Name)>,
//     raycast_cursor_last_pos: &mut ResMut<RaycastCursorLast>,
//     raycast_cursor_pos: &mut ResMut<RaycastCursorPos>,
// ) -> (Option<Entity>, HitType) {
//     if let Some(cursor_ray) = ***cursor_ray {
//         log!(
//             LogType::Editor,
//             LogLevel::Info,
//             LogCategory::Input,
//             "Sending ray at cursor"
//         );

//         let gizmo_filter = |entity: Entity| {
//             query
//                 .get(entity)
//                 .is_ok_and(|(_, gizmo, _, _)| gizmo.is_some())
//         };

//         let icon_filter = |entity: Entity| {
//             query
//                 .get(entity)
//                 .is_ok_and(|(_, gizmo, icon, _)| gizmo.is_none() && icon.is_some())
//         };

//         // Priority 1: Gizmo entities
//         let gizmo_settings = RaycastSettings::default().with_filter(&gizmo_filter);
//         let gizmo_hits = raycast.cast_ray(cursor_ray, &gizmo_settings);

//         if let Some((gizmo_entity, intersection_data)) = gizmo_hits.first() {
//             let current_position = intersection_data.position();
//             if current_position != raycast_cursor_last_pos.position {
//                 raycast_cursor_last_pos.position = raycast_cursor_pos.position;
//                 raycast_cursor_pos.position = current_position;
//             }

//             // Log the name of the entity that was hit
//             if let Ok((_entity, _gizmo, _icon, name)) = query.get(*gizmo_entity) {
//                 log!(
//                     LogType::Editor,
//                     LogLevel::Info,
//                     LogCategory::Input,
//                     "Gizmo ray hit: {}",
//                     name
//                 );
//             }

//             return (Some(*gizmo_entity), HitType::Gizmo);
//         }

//         // Priority 2: Icon entities
//         let icon_settings = RaycastSettings::default().with_filter(&icon_filter);
//         let icon_hits = raycast.cast_ray(cursor_ray, &icon_settings);

//         if let Some((icon_entity, intersection_data)) = icon_hits.first() {
//             let current_position = intersection_data.position();
//             if current_position != raycast_cursor_last_pos.position {
//                 raycast_cursor_last_pos.position = raycast_cursor_pos.position;
//                 raycast_cursor_pos.position = current_position;
//             }

//             // Log the name of the entity that was hit
//             if let Ok((_entity, _gizmo, _icon, name)) = query.get(*icon_entity) {
//                 log!(
//                     LogType::Editor,
//                     LogLevel::Info,
//                     LogCategory::Input,
//                     "Icon ray hit: {}",
//                     name
//                 );
//             }

//             return (Some(*icon_entity), HitType::Icon);
//         }

//         // Priority 3: Mesh entities (unfiltered)
//         let mesh_settings = RaycastSettings::default();
//         let mesh_hits = raycast.cast_ray(cursor_ray, &mesh_settings);

//         if let Some((mesh_entity, intersection_data)) = mesh_hits.first() {
//             let current_position = intersection_data.position();
//             if current_position != raycast_cursor_last_pos.position {
//                 raycast_cursor_last_pos.position = raycast_cursor_pos.position;
//                 raycast_cursor_pos.position = current_position;
//             }

//             // Log the name of the entity that was hit
//             if let Ok((_entity, _gizmo, _icon, name)) = query.get(*mesh_entity) {
//                 log!(
//                     LogType::Editor,
//                     LogLevel::Info,
//                     LogCategory::Input,
//                     "Mesh ray hit: {}",
//                     name
//                 );
//             }

//             return (Some(*mesh_entity), HitType::Mesh);
//         } else {
//             log!(
//                 LogType::Editor,
//                 LogLevel::Info,
//                 LogCategory::Input,
//                 "Ray exists, but couldn't hit anything: {:?}",
//                 cursor_ray
//             );
//             return (None, HitType::Void);
//         }
//     } else {
//         log!(
//             LogType::Editor,
//             LogLevel::Warning,
//             LogCategory::Input,
//             "Cursor ray is None"
//         );
//         return (None, HitType::None);
//     }
// }

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
