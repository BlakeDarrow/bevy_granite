use super::{
    data::NodeTreeTabData,
    hierarchy::{detect_changes, update_hierarchy_data},
    selection::{
        handle_external_selection_change, process_selection_changes, update_tree_click_protection,
        validation::is_valid_drop,
    },
    RequestReparentEntityEvent,
};
use crate::interface::events::RequestRemoveParentsFromEntities;
use crate::interface::{SideDockState, SideTab};
use crate::{
    editor_state::EditorState,
    interface::{tabs::node_tree::data::PendingContextAction, EditorEvents, SetActiveWorld},
};
use bevy::ecs::query::Has;
use bevy::ecs::system::Commands;
use bevy::{
    ecs::query::{Changed, Or},
    prelude::{ChildOf, Entity, EventWriter, Name, Query, Res, ResMut, With},
};
use bevy_granite_core::{
    IdentityData, RequestDespawnBySource, RequestReloadEvent, SpawnSource, TreeHiddenEntity,
};
use bevy_granite_gizmos::{ActiveSelection, GizmoChildren, GizmoMesh, Selected};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

pub fn update_node_tree_tabs_system(
    mut right_dock: ResMut<SideDockState>,
    active_selection: Query<Entity, With<ActiveSelection>>,
    all_selected: Query<Entity, With<Selected>>,
    editor_state: Res<EditorState>,
    hierarchy_query: Query<(
        Entity,
        &Name,
        Option<&ChildOf>,
        Option<&IdentityData>,
        Option<&SpawnSource>,
        (Has<GizmoChildren>, Has<GizmoMesh>, Has<TreeHiddenEntity>),
    )>,
    changed_hierarchy: Query<
        (Has<GizmoChildren>, Has<GizmoMesh>, Has<TreeHiddenEntity>),
        Or<(Changed<Name>, Changed<IdentityData>, Changed<SpawnSource>)>,
    >,
    mut commands: Commands,
    mut editor_events: EditorEvents,
    mut reparent_event_writer: EventWriter<RequestReparentEntityEvent>,
) {
    for (_, tab) in right_dock.dock_state.iter_all_tabs_mut() {
        if let SideTab::NodeTree { ref mut data, .. } = tab {
            let previous_selection = data.active_selection;
            data.active_selection = active_selection.single().ok();
            data.selected_entities = all_selected.iter().collect();
            data.active_scene_file = editor_state.current_file.clone();

            let (entities_changed, data_changed, hierarchy_changed) = if data.filtered_hierarchy {
                let q = hierarchy_query
                    .iter()
                    .filter(|(_, _, _, _, _, a)| !(a.0 || a.1 || a.2))
                    .map(|(a, b, c, d, e, _)| (a, b, c, d, e));
                let c = changed_hierarchy
                    .iter()
                    .any(|filter| !(filter.0 || filter.1 || filter.2));
                detect_changes(q, c, data)
            } else {
                let q = hierarchy_query
                    .iter()
                    .map(|(a, b, c, d, e, _)| (a, b, c, d, e));
                let c = !changed_hierarchy.is_empty();
                detect_changes(q, c, data)
            };

            if entities_changed || data_changed || hierarchy_changed {
                if data.filtered_hierarchy {
                    let q = hierarchy_query
                        .iter()
                        .filter(|(_, _, _, _, _, a)| !(a.0 || a.1 || a.2))
                        .map(|(a, b, c, d, e, _)| (a, b, c, d, e));
                    update_hierarchy_data(data, q, hierarchy_changed);
                } else {
                    let q = hierarchy_query
                        .iter()
                        .map(|(a, b, c, d, e, _)| (a, b, c, d, e));
                    update_hierarchy_data(data, q, hierarchy_changed);
                }
            }

            handle_external_selection_change(data, previous_selection);
            process_selection_changes(data, &mut commands);
            update_tree_click_protection(data);
            handle_drag_drop_events(
                data,
                &mut reparent_event_writer,
                &mut editor_events.remove_parent_entities,
            );
            process_context_actions(data, &mut editor_events, &mut commands);
        }
    }
}

fn handle_drag_drop_events(
    data: &mut crate::interface::tabs::NodeTreeTabData,
    reparent_event_writer: &mut EventWriter<RequestReparentEntityEvent>,
    remove_parents_event_writer: &mut EventWriter<RequestRemoveParentsFromEntities>,
) {
    if let Some(dragged_entities) = data.drag_payload.clone() {
        if let Some(drop_target) = data.drop_target {
            if drop_target == Entity::PLACEHOLDER {
                // Special case: drop on empty space = remove parents
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::UI,
                    "Dropping entities on empty space - removing parents"
                );
                remove_parents_event_writer.write(RequestRemoveParentsFromEntities {
                    entities: dragged_entities,
                });
            } else if is_valid_drop(&dragged_entities, drop_target, &data.hierarchy) {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::UI,
                    "Reparenting {:?} entities to {:?}",
                    dragged_entities.len(),
                    drop_target
                );
                reparent_event_writer.write(RequestReparentEntityEvent {
                    entities: dragged_entities,
                    new_parent: drop_target,
                });
            }

            // Clear drag state after processing
            data.drag_payload = None;
            data.drop_target = None;
        }
    }
}

/// Processes pending context menu actions
fn process_context_actions(
    data: &mut NodeTreeTabData,
    events: &mut EditorEvents,
    commands: &mut Commands,
) {
    for action in data.pending_context_actions.drain(..) {
        match action {
            PendingContextAction::DeleteEntity(entity) => {
                commands.entity(entity).despawn();
            }
            PendingContextAction::SetActiveScene(scene_path) => {
                events.set_active_world.write(SetActiveWorld(scene_path));
            }
            PendingContextAction::ReloadScene(scene_path) => {
                events.reload.write(RequestReloadEvent(scene_path));
            }
            PendingContextAction::DespawnScene(scene_path) => {
                events
                    .despawn_by_source
                    .write(RequestDespawnBySource(scene_path));
            }
        }
    }
}
