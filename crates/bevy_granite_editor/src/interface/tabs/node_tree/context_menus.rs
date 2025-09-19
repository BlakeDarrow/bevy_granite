use super::data::{HierarchyEntry, NodeTreeTabData};
use crate::interface::tabs::node_tree::data::PendingContextAction;
use bevy::prelude::Entity;
use bevy_egui::egui;
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

pub fn show_entity_context_menu(
    _ui: &mut egui::Ui,
    entity: Entity,
    entry: &HierarchyEntry,
    data: &mut NodeTreeTabData,
    response: &egui::Response,
) -> bool {
    let mut menu_shown = false;

    response.context_menu(|ui| {
        menu_shown = true;

        if ui.button("Delete").clicked() {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::UI,
                "Context menu: Delete entity {:?} ('{}')",
                entity,
                entry.name
            );
            data.pending_context_actions
                .push(PendingContextAction::DeleteEntity(entity));

            ui.close();
        }
    });

    menu_shown
}

pub fn show_scene_context_menu(
    _ui: &mut egui::Ui,
    scene_path: &str,
    _entry: &HierarchyEntry,
    data: &mut NodeTreeTabData,
    response: &egui::Response,
) -> bool {
    let mut menu_shown = false;
    let is_active = data
        .active_scene_file
        .as_ref()
        .map_or(false, |active| active == scene_path);

    response.context_menu(|ui| {
        menu_shown = true;

        if !is_active && ui.button("Set Active").clicked() {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::UI,
                "Context menu: Set active scene '{}'",
                scene_path
            );
            data.pending_context_actions
                .push(PendingContextAction::SetActiveScene(
                    scene_path.to_string(),
                ));

            ui.close();
        }

        if ui.button("Reload").clicked() {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::UI,
                "Context menu: Reload scene '{}'",
                scene_path
            );
            data.pending_context_actions
                .push(PendingContextAction::ReloadScene(scene_path.to_string()));

            ui.close();
        }

        if ui.button("Despawn").clicked() {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::UI,
                "Context menu: Despawn scene '{}'",
                scene_path
            );
            data.pending_context_actions
                .push(PendingContextAction::DespawnScene(scene_path.to_string()));

            ui.close();
        }
    });

    menu_shown
}

pub fn handle_context_menu(
    ui: &mut egui::Ui,
    entity: Entity,
    data: &mut NodeTreeTabData,
    button_response: &egui::Response,
) -> bool {
    if let Some(entry) = data.hierarchy.iter().find(|e| e.entity == entity).cloned() {
        if entry.is_dummy_parent {
            return show_scene_context_menu(ui, &entry.name, &entry, data, button_response);
        } else {
            return show_entity_context_menu(ui, entity, &entry, data, button_response);
        }
    }
    false
}

pub fn should_show_context_menu(response: &egui::Response) -> bool {
    response.secondary_clicked()
}
