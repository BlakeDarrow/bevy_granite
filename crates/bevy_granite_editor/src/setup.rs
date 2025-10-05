use crate::editor_state::EditorState;
use bevy::ecs::system::Res;
use bevy_egui::{egui, EguiContexts};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

pub fn editor_info() {
    let mut text: String = Default::default();
    text += "\n\n";

    text += r#"
 $$$$$$\                               $$\   $$\               
$$  __$$\                              \__|  $$ |              
$$ /  \__| $$$$$$\  $$$$$$\  $$$$$$$\  $$\ $$$$$$\    $$$$$$\  
$$ |$$$$\ $$  __$$\ \____$$\ $$  __$$\ $$ |\_$$  _|  $$  __$$\ 
$$ |\_$$ |$$ |  \__|$$$$$$$ |$$ |  $$ |$$ |  $$ |    $$$$$$$$ |
$$ |  $$ |$$ |     $$  __$$ |$$ |  $$ |$$ |  $$ |$$\ $$   ____|
\$$$$$$  |$$ |     \$$$$$$$ |$$ |  $$ |$$ |  \$$$$  |\$$$$$$$\ 
 \______/ \__|      \_______|\__|  \__|\__|   \____/  \_______|
"#;

    text += "\n\nBlake Darrow - 2024, 2025\nVersion: 0.2.1\n\n";

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::System,
        "{}",
        text
    );
}

pub fn setup_ui_style(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut().expect("Egui context is not available");

    let mut style = (*ctx.style()).clone();

    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.item_spacing = egui::vec2(4.0, 4.0);

    ctx.set_style(style);
}

pub fn is_editor_active(editor_state: Res<EditorState>) -> bool {
    editor_state.active
}
