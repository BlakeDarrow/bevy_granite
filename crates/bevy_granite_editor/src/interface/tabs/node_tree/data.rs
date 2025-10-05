use bevy::{
    ecs::message::Message,
    prelude::{Entity, Event},
};

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
    pub expand_to_enabled: bool,  // whether to auto-expand to selected entities
    pub scroll_to_enabled: bool,  // whether to auto-scroll to selected entities
    pub active_selection: Option<Entity>,
    pub selected_entities: Vec<Entity>,
    pub new_selection: Option<Entity>,
    pub additive_selection: bool, // ctrl/cmd
    pub range_selection: bool,    // shift
    pub clicked_via_node_tree: bool,
    pub tree_click_frames_remaining: u8, // Frames to wait before allowing external expansion
    pub hierarchy: Vec<HierarchyEntry>,
    pub should_scroll_to_selection: bool,
    pub scroll_delay_frames: u8, // Frames to wait before scrolling (to allow expansion to render)
    pub previous_active_selection: Option<Entity>,
    pub search_filter: String,
    pub drag_payload: Option<Vec<Entity>>, // Entities being dragged
    pub drop_target: Option<Entity>,       // Entity being dropped onto
    pub active_scene_file: Option<String>, // Currently active scene file path
    pub pending_context_actions: Vec<PendingContextAction>, // Actions from context menus

    // Virtual scrolling fields
    pub virtual_scroll_state: VirtualScrollState,
    pub flattened_tree_cache: Vec<FlattenedTreeNode>,
    pub tree_cache_dirty: bool,
}

/// Virtual scrolling state for performance optimization
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualScrollState {
    pub total_rows: usize,
    pub row_height: f32,
    pub visible_start: usize,
    pub visible_count: usize,
    pub buffer_size: usize,
    pub scroll_offset: f32,
}

impl Default for VirtualScrollState {
    fn default() -> Self {
        Self {
            total_rows: 0,
            row_height: 20.0, // Will be calculated dynamically
            visible_start: 0,
            visible_count: 0, // 0 = auto-calculate based on available height
            buffer_size: 10,
            scroll_offset: 0.0,
        }
    }
}

impl VirtualScrollState {
    /// Sets a fixed number of visible rows (for testing/debugging virtual scrolling)
    pub fn with_fixed_visible_count(mut self, count: usize) -> Self {
        self.visible_count = count;
        self
    }

    /// Returns true if using auto-calculated visible count
    pub fn is_auto_calculated(&self) -> bool {
        self.visible_count == 0
    }
}

/// A flattened representation of a tree node for virtual scrolling
#[derive(Debug, Clone, PartialEq)]
pub struct FlattenedTreeNode {
    pub entity: Entity,
    pub name: String,
    pub entity_type: String,
    pub parent: Option<Entity>,
    pub depth: usize,
    pub is_expanded: bool,
    pub has_children: bool,
    pub is_dummy_parent: bool,
    pub is_preserve_disk: bool,
    pub is_preserve_disk_transform: bool,
}

impl Default for NodeTreeTabData {
    fn default() -> Self {
        Self {
            filtered_hierarchy: true,
            expand_to_enabled: true,
            scroll_to_enabled: true,
            active_selection: None,
            selected_entities: Vec::new(),
            new_selection: None,
            additive_selection: false,
            range_selection: false,
            clicked_via_node_tree: false,
            tree_click_frames_remaining: 0,
            hierarchy: Vec::new(),
            should_scroll_to_selection: false,
            scroll_delay_frames: 0,
            previous_active_selection: None,
            search_filter: String::new(),
            drag_payload: None,
            drop_target: None,
            active_scene_file: None,
            pending_context_actions: Vec::new(),
            virtual_scroll_state: VirtualScrollState::default(),
            flattened_tree_cache: Vec::new(),
            tree_cache_dirty: true,
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
    pub is_dummy_parent: bool,
    pub is_preserve_disk: bool,
    pub is_preserve_disk_transform: bool,
}

/// Events for node tree operations
#[derive(Debug, Clone, Message)]
pub struct RequestReparentEntityEvent {
    pub entities: Vec<Entity>,
    pub new_parent: Entity,
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
            !entities.contains(&entry.entity)
                && super::validation::is_valid_drop(entities, entry.entity, &data.hierarchy)
        });

        let is_invalid_drop_target = data.drag_payload.as_ref().map_or(false, |entities| {
            entities.contains(&entry.entity)
                || entities.iter().any(|&dragged_entity| {
                    super::validation::is_descendant_of(
                        entry.entity,
                        dragged_entity,
                        &data.hierarchy,
                    )
                })
        });

        let is_active_scene = entry.is_dummy_parent
            && data
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

    pub fn from_flattened_node(node: &FlattenedTreeNode, data: &NodeTreeTabData) -> Self {
        let is_selected = data.selected_entities.contains(&node.entity);
        let is_active_selected = Some(node.entity) == data.active_selection;
        let is_being_dragged = data
            .drag_payload
            .as_ref()
            .map_or(false, |entities| entities.contains(&node.entity));

        let is_valid_drop_target = data.drag_payload.as_ref().map_or(false, |entities| {
            !entities.contains(&node.entity)
                && super::validation::is_valid_drop(entities, node.entity, &data.hierarchy)
        });

        let is_invalid_drop_target = data.drag_payload.as_ref().map_or(false, |entities| {
            entities.contains(&node.entity)
                || entities.iter().any(|&dragged_entity| {
                    super::validation::is_descendant_of(
                        node.entity,
                        dragged_entity,
                        &data.hierarchy,
                    )
                })
        });

        let is_active_scene = node.is_dummy_parent
            && data
                .active_scene_file
                .as_ref()
                .map_or(false, |active| active == &node.name);

        Self {
            is_selected,
            is_active_selected,
            is_being_dragged,
            is_valid_drop_target,
            is_invalid_drop_target,
            is_preserve_disk: node.is_preserve_disk,
            is_preserve_disk_transform: node.is_preserve_disk_transform,
            is_dummy_parent: node.is_dummy_parent,
            is_expanded: node.is_expanded,
            has_children: node.has_children,
            is_active_scene,
        }
    }
}
