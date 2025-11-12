use crate::{
    editor_state::{
        dock::{load_dock_state, DockLayoutStr},
        editor::save_editor_settings_from_widget_data,
        EditorState,
    },
    interface::{
        layout::{DockState, SidePanelPosition},
        panels::{BottomDockState, SideDockState, SideTab},
        themes::{SerializableTextStyle, ThemeState},
        EditorEvents, PopupMenuRequestedEvent, PopupType,
    },
    viewport::{DebugRenderer, SelectionRenderer, ViewportState},
};

use bevy::{gizmos::config::GizmoConfigStore, math::Vec2, prelude::ResMut};
use bevy_egui::egui::{self};
use bevy_egui::EguiContexts;
use bevy_granite_core::PromptImportSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum SettingsTab {
    #[default]
    Viewport,
    Interface,
    Import,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct ImportState {
    pub import_settings: PromptImportSettings,

    #[serde(skip)]
    pub changed: bool,
}
impl Default for ImportState {
    fn default() -> Self {
        Self {
            import_settings: PromptImportSettings::default(),
            changed: false,
        }
    }
}

// Currently the truth for settings. No decoupled struct anywhere for this
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct EditorSettingsTabData {
    pub viewport: ViewportState,
    pub scene_light_enabled: bool,
    pub theme_state: ThemeState,
    pub import_state: ImportState,
    pub dock: DockState,
    pub show_help_on_start: bool,

    #[serde(skip)]
    pub save_requested: bool,
}

impl Default for EditorSettingsTabData {
    fn default() -> Self {
        Self {
            save_requested: false,
            theme_state: ThemeState::default(),
            import_state: ImportState::default(),
            scene_light_enabled: false,
            dock: DockState {
                active_tab: SettingsTab::Viewport,
                layout_str: DockLayoutStr::default(),
                store_position_on_close: true,
                side_panel_position: SidePanelPosition::Right,
                changed: true,
            },
            show_help_on_start: true,
            viewport: ViewportState::default(),
        }
    }
}

pub fn update_editor_settings_tab_system(
    mut contexts: EguiContexts,
    mut side_dock: ResMut<SideDockState>,
    mut bottom_dock: ResMut<BottomDockState>,
    mut editor_state: ResMut<EditorState>,
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
    mut prompt_import_settings: ResMut<PromptImportSettings>,
    mut scene_light_state: ResMut<crate::viewport::SceneLightState>,
    mut events: EditorEvents,
) {
    let ctx = contexts.ctx_mut().expect("Egui context to exist");

    let side_dock_clone = side_dock.clone();
    if editor_state.config_loaded && !editor_state.layout_loaded {
        let config = editor_state.config.clone();
        let dock_layout = config.dock.layout_str.clone();
        load_dock_state(&dock_layout, &mut side_dock, &mut bottom_dock);

        // Show help popup on start
        editor_state.layout_loaded = true;
        if editor_state.config.show_help_on_start {
            events.popup.write(PopupMenuRequestedEvent {
                popup: PopupType::Help,
                mouse_pos: Vec2::NAN,
            });
        }
    }

    for (_, tab) in side_dock.dock_state.iter_all_tabs_mut() {
        if let SideTab::EditorSettings { ref mut data, .. } = tab {
            let mut settings_desynced = false;

            if editor_state.config_loaded && editor_state.layout_loaded {
                **data = editor_state.config.clone();
                editor_state.config_loaded = false;
                settings_desynced = true;
            }

            let theme_state = &mut data.theme_state;
            let viewport_state = &mut data.viewport;
            let import_state = &mut data.import_state;

            // Sync scene light between UI and resource
            if data.scene_light_enabled != scene_light_state.enabled {
                scene_light_state.set_enabled(data.scene_light_enabled);
            } else if scene_light_state.enabled != data.scene_light_enabled {
                data.scene_light_enabled = scene_light_state.enabled;
            }

            if theme_state.theme_changed || settings_desynced {
                theme_state.theme.apply_to_context(ctx);
                theme_state.theme_changed = false;
            }

            if import_state.changed || settings_desynced {
                *prompt_import_settings = import_state.import_settings.clone();
                import_state.changed = false;
            }

            if theme_state.font_scale_changed || settings_desynced {
                let mut style = (*ctx.style()).clone();

                for (text_style, font_id) in style.text_styles.iter_mut() {
                    let serializable = SerializableTextStyle::from(text_style.clone());
                    if let Some(&baseline_size) = theme_state.font_baseline.get(&serializable) {
                        font_id.size = (baseline_size * theme_state.font_scale).round();
                    }
                }

                ctx.set_style(style);
                theme_state.font_scale_changed = false;
            }

            if theme_state.spacing_changed || settings_desynced {
                let mut style = (*ctx.style()).clone();
                style.spacing.item_spacing = egui::vec2(theme_state.spacing, theme_state.spacing);
                ctx.set_style(style);
                theme_state.spacing_changed = false;
            }

            if viewport_state.changed || settings_desynced {
                viewport_state.changed = false;
                editor_state.config.viewport = viewport_state.clone();
                let (sel_config, _) = gizmo_config_store.config_mut::<SelectionRenderer>();
                sel_config.line.width = data.viewport.visualizers.selection_line_thickness;
                let (debug_config, _) = gizmo_config_store.config_mut::<DebugRenderer>();
                debug_config.line.width = data.viewport.visualizers.debug_line_thickness;
            }

            if data.save_requested {
                save_editor_settings_from_widget_data(
                    &mut editor_state,
                    data,
                    side_dock_clone,
                    bottom_dock.clone(),
                );
                data.save_requested = false;
            }

            break;
        }
    }
}
