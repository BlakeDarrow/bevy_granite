use super::{
    data::NodeTreeTabData,
    hierarchy::{detect_changes, update_hierarchy_data},
    selection::{
        handle_external_selection_change, process_selection_changes, update_tree_click_protection,
        update_scroll_delay, validation::is_valid_drop,
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
    prelude::{ChildOf, Entity, MessageWriter, Name, Query, Res, ResMut, With, RemovedComponents},
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
        Or<(Changed<Name>, Changed<IdentityData>, Changed<SpawnSource>, Changed<ChildOf>)>,
    >,
    mut removed_child_of: RemovedComponents<ChildOf>,
    mut commands: Commands,
    mut editor_events: EditorEvents,
    mut reparent_event_writer: MessageWriter<RequestReparentEntityEvent>,
) {
    for (_, tab) in right_dock.dock_state.iter_all_tabs_mut() {
        if let SideTab::NodeTree { ref mut data, .. } = tab {
            let previous_selection = data.active_selection;
            data.active_selection = active_selection.single().ok();
            data.selected_entities = all_selected.iter().collect();
            data.active_scene_file = editor_state.current_file.clone();

            let has_changes = !changed_hierarchy.is_empty() || !removed_child_of.is_empty();
            for _ in removed_child_of.read() {}
            
            if !has_changes && !data.hierarchy.is_empty() {
                // No changes
            } else {
                let filtered_entities: Vec<_> = if data.filtered_hierarchy {
                    hierarchy_query
                        .iter()
                        .filter(|(_, _, _, _, _, a)| !(a.0 || a.1 || a.2))
                        .map(|(a, b, c, d, e, _)| (a, b, c, d, e))
                        .collect()
                } else {
                    hierarchy_query
                        .iter()
                        .map(|(a, b, c, d, e, _)| (a, b, c, d, e))
                        .collect()
                };
                
                let (entities_changed, data_changed, hierarchy_changed) = 
                    detect_changes(filtered_entities.iter().cloned(), has_changes, data);

                if entities_changed || data_changed || hierarchy_changed {
                    update_hierarchy_data(data, filtered_entities, hierarchy_changed);
                    data.tree_cache_dirty = true;
                }
            }

            handle_external_selection_change(data, previous_selection);
            process_selection_changes(data, &mut commands);
            update_tree_click_protection(data);
            update_scroll_delay(data);
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
    reparent_event_writer: &mut MessageWriter<RequestReparentEntityEvent>,
    remove_parents_event_writer: &mut MessageWriter<RequestRemoveParentsFromEntities>,
) {
    if let Some(dragged_entities) = data.drag_payload.clone() {
        if let Some(drop_target) = data.drop_target {
            if drop_target == Entity::PLACEHOLDER {
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
                commands.entity(entity).try_despawn();
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
