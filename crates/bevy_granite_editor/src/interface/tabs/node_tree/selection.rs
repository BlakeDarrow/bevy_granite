use super::data::{HierarchyEntry, NodeTreeTabData};
use super::hierarchy::build_visual_order;
use bevy::prelude::Entity;
use bevy_granite_gizmos::selection::events::EntityEvent;
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

/// Validation functions for drag and drop operations
pub mod validation {
    use super::*;

    /// Check if dropping the entities onto the target would create a valid hierarchy
    pub fn is_valid_drop(entities: &[Entity], target: Entity, hierarchy: &[HierarchyEntry]) -> bool {
        // Don't allow dropping onto any of the entities being dragged
        if entities.contains(&target) {
            return false;
        }

        // Don't allow dropping a parent onto any of its descendants
        for &entity in entities {
            if is_descendant_of(target, entity, hierarchy) {
                return false;
            }
        }

        true
    }

    /// Check if `potential_descendant` is a descendant of `ancestor`
    pub fn is_descendant_of(
        potential_descendant: Entity,
        ancestor: Entity,
        hierarchy: &[HierarchyEntry],
    ) -> bool {
        let mut current = potential_descendant;

        while let Some(entry) = hierarchy.iter().find(|e| e.entity == current) {
            if let Some(parent) = entry.parent {
                if parent == ancestor {
                    return true;
                }
                current = parent;
            } else {
                break;
            }
        }

        false
    }
}

/// Handles entity selection logic
pub fn handle_selection(
    entity: Entity,
    name: &str,
    data: &mut NodeTreeTabData,
    additive: bool,
    range: bool,
) {
    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::UI,
        "Tree Node Selected: {:?} ('{}') (additive: {}, range: {})",
        entity,
        name,
        additive,
        range
    );
    data.clicked_via_node_tree = true;
    data.new_selection = Some(entity);
    data.additive_selection = additive;
    data.range_selection = range;
}

/// Processes selection changes and triggers appropriate events
pub fn process_selection_changes(
    data: &mut NodeTreeTabData,
    commands: &mut bevy::ecs::system::Commands,
) {
    if let Some(new_selection) = data.new_selection {
        if data.clicked_via_node_tree {
            if data.range_selection {
                // Range selection: select all between previous_active_selection and new_selection
                if let Some(prev_active) = data.active_selection {
                    perform_range_selection(prev_active, new_selection, data, commands);
                } else {
                    // No previous selection, just select the new one
                    commands.trigger(EntityEvent::Select {
                        target: new_selection,
                        additive: false,
                    });
                }
                data.previous_active_selection = data.active_selection;
                data.active_selection = Some(new_selection);
            } else if data.additive_selection {
                // Ctrl/Cmd (additive): toggle selection
                let already_selected = data.selected_entities.contains(&new_selection);
                if already_selected {
                    commands.trigger(EntityEvent::Deselect {
                        target: new_selection,
                    });
                } else {
                    commands.trigger(EntityEvent::Select {
                        target: new_selection,
                        additive: true,
                    });
                }
                // Always set the clicked entity as active selection
                data.previous_active_selection = data.active_selection;
                data.active_selection = Some(new_selection);
            } else {
                // Normal selection
                commands.trigger(EntityEvent::Select {
                    target: new_selection,
                    additive: false,
                });
                data.previous_active_selection = data.active_selection;
                data.active_selection = Some(new_selection);
            }
            // Set counter to prevent expansion for a few frames while events are processed
            data.tree_click_frames_remaining = 3;
            data.clicked_via_node_tree = false;
        }
    }
    
    // Reset selection state
    data.new_selection = None;
    data.additive_selection = false;
    data.range_selection = false;
}

/// Handles drag and drop state management
pub fn handle_drag_drop(
    response: &bevy_egui::egui::Response,
    entity: Entity,
    data: &mut NodeTreeTabData,
    search_term: &str,
) {
    // Only allow drag/drop when not searching
    if !search_term.is_empty() {
        return;
    }

    // Handle drag start
    if response.drag_started() {
        let entities_to_drag = if data.selected_entities.contains(&entity) {
            data.selected_entities.clone()
        } else {
            vec![entity]
        };

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::UI,
            "Drag started: {:?} entities",
            entities_to_drag.len()
        );

        data.drag_payload = Some(entities_to_drag);
    }

    // Handle drop detection when mouse is released
    if data.drag_payload.is_some() && response.ctx.input(|i| i.pointer.any_released()) {
        if response.hovered() {
            // Valid drop target
            if let Some(ref dragged_entities) = data.drag_payload {
                if validation::is_valid_drop(dragged_entities, entity, &data.hierarchy) {
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::UI,
                        "Valid drop target: {:?}",
                        entity
                    );
                    data.drop_target = Some(entity);
                }
            }
        }
    }
}

/// Expands the tree to show the path to a specific entity
pub fn expand_to_entity(hierarchy: &mut Vec<HierarchyEntry>, target_entity: Entity) {
    // Find the target
    let mut ancestors = Vec::new();
    let mut current_parent = hierarchy
        .iter()
        .find(|entry| entry.entity == target_entity)
        .and_then(|entry| entry.parent);

    // Walk up the hierarchy to collect ancestors
    while let Some(parent_entity) = current_parent {
        ancestors.push(parent_entity);
        current_parent = hierarchy
            .iter()
            .find(|entry| entry.entity == parent_entity)
            .and_then(|entry| entry.parent);
    }

    // Expand all ancestors
    for ancestor in ancestors {
        if let Some(entry) = hierarchy.iter_mut().find(|e| e.entity == ancestor) {
            entry.is_expanded = true;
        }
    }
}

/// Handles external selection changes (from gizmos, etc.)
pub fn handle_external_selection_change(
    data: &mut NodeTreeTabData,
    previous_selection: Option<Entity>,
) {
    if let Some(new_active) = data.active_selection {
        if previous_selection != Some(new_active)
            && !data.clicked_via_node_tree
            && data.tree_click_frames_remaining == 0
        {
            // Auto-expand and scroll for any external selection change
            expand_to_entity(&mut data.hierarchy, new_active);
            data.should_scroll_to_selection = true;

            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::UI,
                "External selection detected - expanding to entity {:?}",
                new_active
            );
        } else {
            // Prevent scroll/expand for user clicks or no change
            data.should_scroll_to_selection = false;
        }
    }
}

/// Decrements the tree click frame counter
pub fn update_tree_click_protection(data: &mut NodeTreeTabData) {
    if data.tree_click_frames_remaining > 0 {
        data.tree_click_frames_remaining -= 1;
    }
}

/// Performs range selection between two entities in the visual order
fn perform_range_selection(
    start_entity: Entity,
    end_entity: Entity,
    data: &mut NodeTreeTabData,
    commands: &mut bevy::ecs::system::Commands,
) {
    // Build the visual order of entities as they appear in the tree
    let visual_order = build_visual_order(&data.hierarchy);
    
    // Find the indices of start and end entities
    let start_index = visual_order.iter().position(|&e| e == start_entity);
    let end_index = visual_order.iter().position(|&e| e == end_entity);
    
    if let (Some(start_idx), Some(end_idx)) = (start_index, end_index) {
        // Get the range (handle both directions)
        let min_idx = start_idx.min(end_idx);
        let max_idx = start_idx.max(end_idx);
        
        // Clear existing selection first
        commands.trigger(EntityEvent::Select {
            target: visual_order[min_idx],
            additive: false,
        });
        
        // Select all entities in the range
        for i in (min_idx + 1)..=max_idx {
            commands.trigger(EntityEvent::Select {
                target: visual_order[i],
                additive: true,
            });
        }
        
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::UI,
            "Range selection: selected {} entities from index {} to {}",
            max_idx - min_idx + 1,
            min_idx,
            max_idx
        );
    } else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::UI,
            "Could not find start_entity {:?} or end_entity {:?} in visual order for range selection",
            start_entity,
            end_entity
        );
    }
}