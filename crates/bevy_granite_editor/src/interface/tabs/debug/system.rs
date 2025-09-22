use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::platform::collections::HashMap;
use bevy::prelude::{Entity, Name, Query, Res, ResMut, With};
use bevy_granite_core::{AvailableEditableMaterials, IdentityData, SpawnSource, UserInput};
use bevy_granite_gizmos::{ActiveSelection, Selected};

use super::{ActiveObjectDetails, SelectionInfo};
use crate::{
    editor_state::EditorState,
    interface::{BottomDockState, BottomTab},
};
pub fn update_debug_tab_ui_system(
    mut bottom_dock: ResMut<BottomDockState>,
    selection_query: Query<Entity, With<Selected>>,
    active_selection_query: Query<Entity, With<ActiveSelection>>,
    available_materials: Res<AvailableEditableMaterials>,
    user_input: Res<UserInput>,
    editor_state: Res<EditorState>,
    entity_query: Query<&Name>,
    identity_query: Query<&IdentityData>,
    spawn_source_query: Query<&SpawnSource>,
    diagnostics: Res<DiagnosticsStore>,
) {
    for (_, tab) in bottom_dock.dock_state.iter_all_tabs_mut() {
        if let BottomTab::Debug { ref mut data, .. } = tab {
            let active_selection = if let Some(active_entity) = active_selection_query.iter().next()
            {
                let name = entity_query.get(active_entity).ok().map(|n| n.to_string());
                (Some(active_entity), name)
            } else {
                (None, None)
            };

            let active_object_details =
                if let Some(active_entity) = active_selection_query.iter().next() {
                    let name = entity_query.get(active_entity).ok().map(|n| n.to_string());
                    let identity_data = identity_query.get(active_entity).ok().cloned();
                    let spawned_from = spawn_source_query.get(active_entity).ok();

                    ActiveObjectDetails {
                        entity: Some(active_entity),
                        name,
                        identity_data,
                        spawned_from: spawned_from.cloned(),
                    }
                } else {
                    ActiveObjectDetails::default()
                };

            let user_input = user_input.clone();
            data.user_input = user_input;

            let selected_entities: Vec<Entity> = selection_query.iter().collect();
            let selection = if selected_entities.is_empty() {
                None
            } else {
                let mut map = HashMap::new();
                for entity in selected_entities {
                    let name = entity_query.get(entity).ok().map(|n| n.to_string());
                    map.insert(entity, name);
                }
                Some(map)
            };

            data.available_materials = available_materials.clone();

            if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(value) = fps.value() {
                    let fps_value_display = format!("Current FPS: {:.0}", value);
                    data.fps_info.0 = fps_value_display;
                }
                if let Some(average) = fps.average() {
                    let fps_average_display = format!("Average FPS: {:.0}", average);
                    data.fps_info.1 = fps_average_display;
                }
            }

            if let Some(file) = &editor_state.current_file {
                data.current_file = file.to_string();
            } else {
                data.current_file = "None".to_string();
            }
            data.selection_info = SelectionInfo {
                active_selection,
                selection,
            };
            data.active_object_details = active_object_details;
        }
    }
}
