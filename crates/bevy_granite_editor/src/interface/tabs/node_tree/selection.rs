use super::data::{HierarchyEntry, NodeTreeTabData};
use super::hierarchy::build_visual_order;
use bevy::prelude::Entity;
use bevy_granite_gizmos::selection::events::EntityEvents;
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

/// Validation functions for drag and drop operations
pub mod validation {
    use super::*;
    pub fn is_valid_drop(
        entities: &[Entity],
        target: Entity,
        hierarchy: &[HierarchyEntry],
    ) -> bool {
        if entities.contains(&target) {
            return false;
        }

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
                // Range selection
                if let Some(prev_active) = data.active_selection {
                    perform_range_selection(prev_active, new_selection, data, commands);
                } else {
                    commands.trigger(EntityEvents::Select {
                        target: new_selection,
                        additive: false,
                    });
                }
                data.previous_active_selection = data.active_selection;
                data.active_selection = Some(new_selection);
            } else if data.additive_selection {
                // Toggle selection
                let already_selected = data.selected_entities.contains(&new_selection);
                if already_selected {
                    commands.trigger(EntityEvents::Deselect {
                        target: new_selection,
                    });
                } else {
                    commands.trigger(EntityEvents::Select {
                        target: new_selection,
                        additive: true,
                    });
                }
                data.previous_active_selection = data.active_selection;
                data.active_selection = Some(new_selection);
            } else {
                // Normal selection
                commands.trigger(EntityEvents::Select {
                    target: new_selection,
                    additive: false,
                });
                data.previous_active_selection = data.active_selection;
                data.active_selection = Some(new_selection);
            }
            data.tree_click_frames_remaining = 3;
            data.clicked_via_node_tree = false;
        }
    }

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
    if !search_term.is_empty() {
        return;
    }

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

    if data.drag_payload.is_some() && response.ctx.input(|i| i.pointer.any_released()) {
        if response.hovered() {
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
pub fn expand_to_entity(data: &mut NodeTreeTabData, target_entity: Entity) {
    let mut ancestors = Vec::new();
    let mut current_parent = data
        .hierarchy
        .iter()
        .find(|entry| entry.entity == target_entity)
        .and_then(|entry| entry.parent);

    while let Some(parent_entity) = current_parent {
        ancestors.push(parent_entity);
        current_parent = data
            .hierarchy
            .iter()
            .find(|entry| entry.entity == parent_entity)
            .and_then(|entry| entry.parent);
    }

    let mut cache_needs_update = false;
    for ancestor in ancestors {
        if let Some(entry) = data.hierarchy.iter_mut().find(|e| e.entity == ancestor) {
            if !entry.is_expanded {
                entry.is_expanded = true;
                cache_needs_update = true;
            }
        }
    }

    if cache_needs_update {
        data.tree_cache_dirty = true;
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
            // Handle auto-expand
            if data.expand_to_enabled {
                expand_to_entity(data, new_active);
            }

            if data.scroll_to_enabled {
                if data.expand_to_enabled {
                    data.scroll_delay_frames = 2;
                    data.should_scroll_to_selection = false; // Will be set to true when delay expires
                } else {
                    // Scroll immediately if not expanding
                    data.should_scroll_to_selection = true;
                    data.scroll_delay_frames = 0;
                }
            } else {
                data.should_scroll_to_selection = false;
                data.scroll_delay_frames = 0;
            }
        } else {
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

/// Updates the scroll delay counter and activates scrolling when ready
pub fn update_scroll_delay(data: &mut NodeTreeTabData) {
    if data.scroll_delay_frames > 0 {
        data.scroll_delay_frames -= 1;
        if data.scroll_delay_frames == 0 {
            data.should_scroll_to_selection = true;
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::UI,
                "Scroll delay expired - activating scroll to selection"
            );
        }
    }
}

/// Performs range selection between two entities in the visual order
fn perform_range_selection(
    start_entity: Entity,
    end_entity: Entity,
    data: &mut NodeTreeTabData,
    commands: &mut bevy::ecs::system::Commands,
) {
    let visual_order = build_visual_order(&data.hierarchy);
    let start_index = visual_order.iter().position(|&e| e == start_entity);
    let end_index = visual_order.iter().position(|&e| e == end_entity);

    if let (Some(start_idx), Some(end_idx)) = (start_index, end_index) {
        let min_idx = start_idx.min(end_idx);
        let max_idx = start_idx.max(end_idx);

        commands.trigger(EntityEvents::Select {
            target: start_entity,
            additive: true,
        });

        for i in min_idx..=max_idx {
            commands.trigger(EntityEvents::Select {
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
