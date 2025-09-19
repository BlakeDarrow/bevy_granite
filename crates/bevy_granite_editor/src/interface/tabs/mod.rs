pub mod debug;
pub mod editor_settings;
pub mod entity_editor;
pub mod events;
pub mod log;
pub mod node_tree;

pub use debug::{debug_tab_ui, update_debug_tab_ui_system, DebugTabData};
pub use editor_settings::{update_editor_settings_tab_system, EditorSettingsTabData, SettingsTab};
pub use entity_editor::{
    handle_material_deletion_system, update_entity_editor_tab_system, update_entity_with_new_components_system,
    update_entity_with_new_identity_system, update_entity_with_new_transform_system,
    update_material_handle_system, EntityEditorTabData,
};
pub use events::{events_tab_ui, send_queued_events_system, EventsTabData};
pub use log::{log_tab_ui, update_log_tab_system, LogTabData};
pub use node_tree::{update_node_tree_tabs_system, NodeTreeTabData, RequestReparentEntityEvent};
