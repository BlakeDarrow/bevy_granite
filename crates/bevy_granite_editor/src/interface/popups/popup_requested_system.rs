use bevy::{
    ecs::{
        message::{MessageReader, MessageWriter},
        query::With,
        system::{Query, ResMut},
    },
    math::Vec2,
    prelude::Resource,
    window::{PrimaryWindow, Window},
};
use bevy_egui::EguiContexts;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

use crate::{
    editor_state::EditorState,
    interface::{
        popups::{add_entity_ui, help_ui, relationship_ui},
        EditorEvents, PopupMenuRequestedEvent, UserRequestGraniteTypeViaPopup,
    },
};

#[derive(Debug, Clone)]
pub enum PopupType {
    AddRelationship,
    AddEntity,
    Help,
}

#[derive(Default, Resource)]
pub struct PopupState {
    pub active_popup: Option<PopupType>,
    pub popup_position: Vec2,
}

pub fn handle_popup_requests_system(
    mut popup_reader: MessageReader<PopupMenuRequestedEvent>,
    mut popup_state: ResMut<PopupState>,
) {
    for PopupMenuRequestedEvent { popup, mouse_pos } in popup_reader.read() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::UI,
            "Popup menu requested: {:?}",
            popup
        );
        popup_state.active_popup = Some(popup.clone());
        popup_state.popup_position = *mouse_pos;
    }
}

pub fn show_active_popups_system(
    mut contexts: EguiContexts,
    mut popup_state: ResMut<PopupState>,
    events: EditorEvents,
    entity_add_writer: MessageWriter<UserRequestGraniteTypeViaPopup>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    editor_state: ResMut<EditorState>,
) {
    if let Some(popup_type) = &popup_state.active_popup {
        let should_close = match popup_type {
            PopupType::AddEntity => {
                add_entity_ui(&mut contexts, popup_state.popup_position, entity_add_writer)
            }
            PopupType::AddRelationship => {
                relationship_ui(&mut contexts, popup_state.popup_position, events)
            }
            PopupType::Help => {
                if let Ok(window) = window_query.single() {
                    help_ui(&mut contexts, window, editor_state)
                } else {
                    false
                }
            }
        };

        if should_close {
            popup_state.active_popup = None;
        }
    }
}
