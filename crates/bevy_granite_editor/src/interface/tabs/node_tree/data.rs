use bevy::prelude::{Entity, Event};

/// Pending actions from context menus to be processed by the system
#[derive(Debug, Clone, PartialEq)]
pub enum PendingContextAction {
    DeleteEntity(Entity),
    SetActiveScene(String),
    ReloadScene(String),
    DespawnScene(String),
}

/// Core data structures for the node tree system
#[derive(Debug, Clone, PartialEq)]
pub struct NodeTreeTabData {
    pub filtered_hierarchy: bool, // whether the hierarchy shows all entities or hides editor related ones
    pub active_selection: Option<Entity>,
    pub selected_entities: Vec<Entity>,
    pub new_selection: Option<Entity>,
    pub additive_selection: bool, // ctrl/cmd
    pub range_selection: bool,    // shift
    pub clicked_via_node_tree: bool,
    pub tree_click_frames_remaining: u8, // Frames to wait before allowing external expansion
    pub hierarchy: Vec<HierarchyEntry>,
    pub should_scroll_to_selection: bool,
    pub previous_active_selection: Option<Entity>,
    pub search_filter: String,
    pub drag_payload: Option<Vec<Entity>>, // Entities being dragged
    pub drop_target: Option<Entity>,       // Entity being dropped onto
    pub active_scene_file: Option<String>, // Currently active scene file path
    pub pending_context_actions: Vec<PendingContextAction>, // Actions from context menus
}

impl Default for NodeTreeTabData {
    fn default() -> Self {
        Self {
            filtered_hierarchy: true,
            active_selection: None,
            selected_entities: Vec::new(),
            new_selection: None,
            additive_selection: false,
            range_selection: false,
            clicked_via_node_tree: false,
            tree_click_frames_remaining: 0,
            hierarchy: Vec::new(),
            should_scroll_to_selection: false,
            previous_active_selection: None,
            search_filter: String::new(),
            drag_payload: None,
            drop_target: None,
            active_scene_file: None,
            pending_context_actions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HierarchyEntry {
    pub entity: Entity,
    pub name: String,
    pub entity_type: String,
    pub parent: Option<Entity>,
    pub is_expanded: bool,
    pub is_dummy_parent: bool, // True if this is a file-based grouping dummy parent
    pub is_preserve_disk: bool, // True if entity has SaveSettings::PreserveDiskFull
    pub is_preserve_disk_transform: bool, // True if entity has SaveSettings::PreserveDiskTransform
}

/// Events for node tree operations
#[derive(Debug, Clone, Event)]
pub struct RequestReparentEntityEvent {
    pub entities: Vec<Entity>, // All entities to reparent (preserving internal relationships)
    pub new_parent: Entity,    // The target parent entity
}

/// Visual state for rendering a single tree row
#[derive(Debug, Clone)]
pub struct RowVisualState {
    pub is_selected: bool,
    pub is_active_selected: bool,
    pub is_being_dragged: bool,
    pub is_valid_drop_target: bool,
    pub is_invalid_drop_target: bool,
    pub is_preserve_disk: bool,
    pub is_preserve_disk_transform: bool,
    pub is_dummy_parent: bool,
    pub is_expanded: bool,
    pub has_children: bool,
    pub is_active_scene: bool,
}

impl RowVisualState {
    pub fn from_hierarchy_entry(
        entry: &HierarchyEntry,
        data: &NodeTreeTabData,
        has_children: bool,
    ) -> Self {
        let is_selected = data.selected_entities.contains(&entry.entity);
        let is_active_selected = Some(entry.entity) == data.active_selection;
        let is_being_dragged = data
            .drag_payload
            .as_ref()
            .map_or(false, |entities| entities.contains(&entry.entity));
        
        let is_valid_drop_target = data.drag_payload.as_ref().map_or(false, |entities| {
            !entities.contains(&entry.entity) && super::validation::is_valid_drop(entities, entry.entity, &data.hierarchy)
        });
        
        let is_invalid_drop_target = data.drag_payload.as_ref().map_or(false, |entities| {
            entities.contains(&entry.entity)
                || entities
                    .iter()
                    .any(|&dragged_entity| super::validation::is_descendant_of(entry.entity, dragged_entity, &data.hierarchy))
        });

        let is_active_scene = entry.is_dummy_parent && data
            .active_scene_file
            .as_ref()
            .map_or(false, |active| active == &entry.name);

        Self {
            is_selected,
            is_active_selected,
            is_being_dragged,
            is_valid_drop_target,
            is_invalid_drop_target,
            is_preserve_disk: entry.is_preserve_disk,
            is_preserve_disk_transform: entry.is_preserve_disk_transform,
            is_dummy_parent: entry.is_dummy_parent,
            is_expanded: entry.is_expanded,
            has_children,
            is_active_scene,
        }
    }
}