use bevy::{
    app::PostStartup,
    ecs::{resource::Resource, schedule::IntoScheduleConfigs},
    prelude::{App, Plugin, Res, ResMut, Startup, Update},
};

use super::editor::update_editor_vis_system;
use crate::{
    editor_state::{
        load_editor_settings_toml, save_dock_on_window_close_system, update_active_world_system,
    },
    interface::EditorSettingsTabData,
    setup::is_editor_active,
};
use bevy_granite_gizmos::GizmoVisibilityState;

#[derive(Resource, Clone)]
pub struct EditorState {
    pub active: bool,
    pub default_world: String,
    pub current_file: Option<String>,
    pub config_path: String,
    pub config: EditorSettingsTabData,

    pub config_loaded: bool,
    pub layout_loaded: bool,

    /// Track all loaded sources (world files/paths) that have entities spawned
    pub loaded_sources: std::collections::HashSet<String>,
}

pub struct ConfigPlugin {
    pub editor_active: bool,
    pub default_world: String,
}

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Resources
            //
            .insert_resource(EditorState {
                active: self.editor_active,
                default_world: self.default_world.clone(),
                current_file: None,

                config_path: "config/editor.toml".to_string(),
                config: EditorSettingsTabData::default(),
                config_loaded: false,
                layout_loaded: false,
                loaded_sources: std::collections::HashSet::new(),
            })
            //
            // Systems
            //
            .add_systems(Startup, sync_initial_gizmo_state)
            .add_systems(PostStartup, load_editor_settings_toml)
            .add_systems(Update, update_active_world_system.run_if(is_editor_active))
            .add_systems(Update, save_dock_on_window_close_system)
            .add_systems(Update, update_editor_vis_system);
    }
}

/// This ensures that if the editor starts with active: false, the gizmos are also disabled
fn sync_initial_gizmo_state(
    editor_state: Res<EditorState>,
    mut gizmo_state: ResMut<GizmoVisibilityState>,
) {
    gizmo_state.active = editor_state.active;
}
