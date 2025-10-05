use bevy::{
    math::Vec2,
    prelude::{Children, Commands, Entity, Query, Res},
};
use bevy_granite_core::{
    entities::SaveSettings, RequestLoadEvent, RequestReloadEvent, RequestSaveEvent, UserInput,
};
use bevy_granite_gizmos::{selection::events::EntityEvents, Selected};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};
use native_dialog::FileDialog;

use crate::{
    editor_state::EditorState,
    interface::{
        events::{
            PopupMenuRequestedEvent, RequestCameraEntityFrame, RequestEditorToggle,
            RequestToggleCameraSync,
        },
        popups::PopupType,
        EditorEvents,
    },
};

pub fn shortcuts_system(
    mut commands: Commands,
    input: Res<UserInput>,
    query: Query<(Entity, &Selected, Option<&Children>)>,
    mut events: EditorEvents,
    editor_state: Res<EditorState>,
) {
    handle_shortcuts(&input, &editor_state, &mut commands, &query, &mut events);
}

fn handle_shortcuts(
    input: &UserInput,
    editor_state: &EditorState,
    commands: &mut Commands,
    query: &Query<(Entity, &Selected, Option<&Children>)>,
    events: &mut EditorEvents,
) {
    // F2
    // Toggle editor on/off
    if input.key_f2.just_pressed {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Toggling editor"
        );
        events.toggle_editor.write(RequestEditorToggle);
    }

    if !editor_state.active {
        return;
    }

    // F3
    // sync cam
    if input.key_f3.just_pressed {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Toggling camera sync"
        );
        events.toggle_cam_sync.write(RequestToggleCameraSync);
    }

    // Delete Key
    // Delete Active Entity
    if input.key_delete.just_pressed {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Deleting active entity"
        );
        for (entity, _, children) in query.iter() {
            if let Some(children) = children {
                for &child in children.iter() {
                    commands.entity(child).try_despawn();
                }
            }
            commands.entity(entity).try_despawn();
        }
    }

    // F key
    // Frame selected entity
    if input.key_f.just_pressed && !input.mouse_over_egui {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Framing selected entity"
        );
        events.frame.write(RequestCameraEntityFrame);
    }

    // U key
    // Deselect all
    if input.key_u.just_pressed && !input.mouse_over_egui {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Deselecting all entities"
        );
        commands.trigger(EntityEvents::DeselectAll);
    }

    // Shft-A
    // Add Entity
    if input.shift_left.pressed
        && input.key_a.just_pressed
        && !input.mouse_over_egui
        && !input.mouse_right.any
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Opening Add Entity popup"
        );
        events.popup.write(PopupMenuRequestedEvent {
            popup: PopupType::AddEntity,
            mouse_pos: input.mouse_pos,
        });
    }

    // F1
    // Help
    if input.key_f1.just_pressed && !input.mouse_over_egui && !input.mouse_right.any {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Opening help popup"
        );
        events.popup.write(PopupMenuRequestedEvent {
            popup: PopupType::Help,
            mouse_pos: Vec2::NAN,
        });
    }

    // Ctrl-O
    // Load
    if input.ctrl_left.pressed && input.key_o.just_pressed && !input.mouse_right.any {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Opening load world dialog"
        );
        if let Some(path) = FileDialog::new()
            .add_filter("Granite Scene", &["scene"])
            .show_open_single_file()
            .unwrap()
        {
            events.load.write(RequestLoadEvent(
                path.display().to_string(),
                SaveSettings::Runtime,
                None,
            ));
        };
    }

    // Ctrl-S
    // Save all sources
    if input.ctrl_left.pressed
        && input.key_s.just_pressed
        && !input.mouse_right.any
        && !input.mouse_left.any
    {
        let loaded = &editor_state.loaded_sources;
        if !loaded.is_empty() {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Input,
                "(shortcut) Saving loaded worlds: {:?}",
                loaded
            );
            for (index, source) in loaded.iter().enumerate() {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Input,
                    "(shortcut) Sending save request #{} for source: '{}'",
                    index + 1,
                    source
                );
                events.save.write(RequestSaveEvent(source.to_string()));
            }
        } else {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::Input,
                "(shortcut) No save path set, cannot save world"
            );
        }
    }

    // Reload loaded worlds
    // Despawn entities and reload world
    if input.ctrl_left.pressed
        && input.key_r.just_pressed
        && !input.mouse_right.any
        && !input.mouse_left.any
    {
        if let Some(current_file) = &editor_state.current_file {
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Input,
                "(shortcut) Reloading world from file"
            );
            events
                .reload
                .write(RequestReloadEvent(current_file.to_string()));
        } else {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::Input,
                "(shortcut) No current file set, cannot reload world"
            );
        }
    }

    // Shft-P
    // Relationship menu
    if input.shift_left.pressed
        && input.key_p.just_pressed
        && !input.mouse_over_egui
        && !input.mouse_right.any
    {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Opening Add Relationship popup"
        );
        events.popup.write(PopupMenuRequestedEvent {
            popup: PopupType::AddRelationship,
            mouse_pos: input.mouse_pos,
        });
    }
}
