use crate::interface::{
    events::{
        RequestNewParent, RequestRemoveChildren, RequestRemoveParents,
        RequestRemoveParentsFromEntities,
    },
    tabs::RequestReparentEntityEvent,
};
use bevy::{
    ecs::{
        entity::Entity,
        message::MessageReader,
        query::{With, Without},
        system::{Commands, Query},
    },
    prelude::{ChildOf, Children},
    transform::commands::BuildChildrenTransformExt,
};
use bevy_granite_core::IconProxy;
use bevy_granite_gizmos::{selection::events::EntityEvents, ActiveSelection, Selected};
use bevy_granite_logging::*;

pub fn parent_from_node_tree_system(
    mut parent_request: MessageReader<RequestReparentEntityEvent>,
    mut commands: Commands,
) {
    for request in parent_request.read() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Re-parenting {} entities to Entity {:?}",
            request.entities.len(),
            request.new_parent
        );

        // Set the new parent for all entities in the request
        for &entity in &request.entities {
            commands
                .entity(entity)
                .set_parent_in_place(request.new_parent);

            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Entity,
                "Entity {:?} reparented to {:?}",
                entity,
                request.new_parent
            );
        }

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Node tree reparenting completed"
        );
    }
}

pub fn parent_system(
    mut parent_request: MessageReader<RequestNewParent>,
    active_selection: Query<Entity, With<ActiveSelection>>,
    selection: Query<Entity, (With<Selected>, Without<ActiveSelection>)>,
    mut commands: Commands,
) {
    for _request in parent_request.read() {
        if let Ok(active_entity) = active_selection.single() {
            for selected_entity in selection.iter() {
                commands
                    .entity(selected_entity)
                    .set_parent_in_place(active_entity);
            }
            log!(
                LogType::Editor,
                LogLevel::OK,
                LogCategory::Entity,
                "New parent applied to selection"
            );
        } else {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::Entity,
                "No active entity to set as parent!"
            );
        }
    }
}

pub fn parent_removal_system(
    mut parent_request: MessageReader<RequestRemoveParents>,
    active_selection: Query<Entity, (With<ActiveSelection>, With<ChildOf>)>,
    selection: Query<Entity, (With<Selected>, With<ChildOf>)>,
    mut commands: Commands,
) {
    for _request in parent_request.read() {
        for entity in selection.iter() {
            commands.entity(entity).remove_parent_in_place();
        }

        for entity in active_selection.iter() {
            commands.entity(entity).remove_parent_in_place();
        }

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::Entity,
            "Parents removed from selection"
        );
    }
}

pub fn parent_removal_from_entities_system(
    mut parent_request: MessageReader<RequestRemoveParentsFromEntities>,
    mut commands: Commands,
) {
    for request in parent_request.read() {
        for &entity in &request.entities {
            commands.entity(entity).remove_parent_in_place();

            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Entity,
                "Parent removed from Entity {:?}",
                entity
            );
        }

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::Entity,
            "Parents removed from {} entities",
            request.entities.len()
        );
    }
}

pub fn child_removal_system(
    mut child_request: MessageReader<RequestRemoveChildren>,
    selection: Query<Entity, (With<Selected>, With<Children>)>,
    active_selection: Query<Entity, (With<ActiveSelection>, With<ChildOf>)>,
    children_query: Query<&Children>,
    icon_proxy_query: Query<(), With<IconProxy>>,
    mut commands: Commands,
) {
    for _request in child_request.read() {
        for entity in active_selection.iter() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    if icon_proxy_query.get(child).is_err() {
                        commands.entity(child).remove_parent_in_place();
                    }
                }
            }
        }

        for entity in selection.iter() {
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    if icon_proxy_query.get(child).is_err() {
                        commands.entity(child).remove_parent_in_place();
                    }
                }
            }
        }

        // Gizmo has weird issue where it stays in place when children are removed
        // so we deselect all entities to ensure it updates correctly
        commands.trigger(EntityEvents::DeselectAll);

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::Entity,
            "Child removed from selection"
        );
    }
}
