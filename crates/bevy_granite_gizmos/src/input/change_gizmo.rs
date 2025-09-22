use crate::gizmos::{NewGizmoType, GizmoType};
use bevy::{
    ecs::system::{Res, ResMut},
    input::keyboard::KeyCode,
};
use bevy_granite_core::{InputTypes, UserInput};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

pub fn watch_gizmo_change(user_input: Res<UserInput>, mut selected_gizmo: ResMut<NewGizmoType>) {
    // By grabbing the list of inputs, we can ensure only that key is pressed
    let allow_transform = user_input.current_button_inputs.len() == 1
        && user_input.current_button_inputs[0] == InputTypes::Button(KeyCode::KeyW)
        && !user_input.mouse_over_egui;

    let allow_rotate = user_input.current_button_inputs.len() == 1
        && user_input.current_button_inputs[0] == InputTypes::Button(KeyCode::KeyE)
        && !user_input.mouse_over_egui;

    let allow_pointer = user_input.current_button_inputs.len() == 1
        && user_input.current_button_inputs[0] == InputTypes::Button(KeyCode::KeyQ)
        && !user_input.mouse_over_egui;

    if allow_transform && !matches!(**selected_gizmo, GizmoType::Transform) {
        **selected_gizmo = GizmoType::Transform;
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Toggling gizmo to Transform"
        );
    }

    if allow_rotate && !matches!(**selected_gizmo, GizmoType::Rotate) {
        **selected_gizmo = GizmoType::Rotate;
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Toggling gizmo to Rotate"
        );
    }

    if allow_pointer && !matches!(**selected_gizmo, GizmoType::Pointer) {
        **selected_gizmo = GizmoType::Pointer;
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "(shortcut) Toggling gizmo to Pointer"
        );
    }
}
