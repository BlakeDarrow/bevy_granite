use crate::{
    editor_state::{DockLayoutStr, EditorState},
    get_interface_config_float,
    interface::{
        layout::top_bar::top_bar_ui,
        panels::{
            bottom_panel::{BottomDockState, BottomTabViewer},
            right_panel::{SideDockState, SideTabViewer},
        },
        EditorEvents, SettingsTab,
    },
    viewport::{EditorViewportCamera, ViewportCameraState},
};

use bevy::{
    camera::{Camera, Camera3d, RenderTarget},
    ecs::system::{Commands, Query},
    prelude::{Entity, Name, Res, ResMut},
};
use bevy_egui::{egui, EguiContexts};
use bevy_granite_core::{UICamera, UserInput};
use bevy_granite_gizmos::GizmoCamera;
use egui_dock::DockArea;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum SidePanelPosition {
    Left,
    #[default]
    Right,
}

impl SidePanelPosition {
    pub fn all() -> Vec<Self> {
        vec![Self::Left, Self::Right]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DockState {
    #[serde(skip)]
    pub active_tab: SettingsTab,

    pub store_position_on_close: bool,
    pub side_panel_position: SidePanelPosition,
    pub layout_str: DockLayoutStr,

    #[serde(skip)]
    pub changed: bool,
}

pub fn dock_ui_system(
    mut contexts: EguiContexts,
    mut side_dock: ResMut<SideDockState>,
    mut bottom_dock: ResMut<BottomDockState>,
    mut events: EditorEvents,
    editor_state: Res<EditorState>,
    user_input: Res<UserInput>,
    mut commands: Commands,
    camera_query: Query<(
        Entity,
        Option<&Name>,
        &Camera,
        Option<&Camera3d>,
        Option<&EditorViewportCamera>,
        Option<&UICamera>,
        Option<&GizmoCamera>,
    )>,
    viewport_camera_state: Res<ViewportCameraState>,
) {
    let mut camera_options: Vec<(Entity, String)> = camera_query
        .iter()
        .filter_map(
            |(entity, name, camera, camera3d, editor_camera, ui_camera, gizmo_camera)| {
                if camera3d.is_some()
                    && editor_camera.is_none()
                    && ui_camera.is_none()
                    && gizmo_camera.is_none()
                    && matches!(camera.target, RenderTarget::Window(_))
                {
                    let label = name
                        .map(|n| n.as_str().to_string())
                        .unwrap_or_else(|| format!("Camera {}", entity.index()));
                    Some((entity, label))
                } else {
                    None
                }
            },
        )
        .collect();
    camera_options.sort_by(|a, b| a.1.cmp(&b.1));

    let ctx = contexts.ctx_mut().expect("Egui context to exist");
    let screen_rect = ctx.screen_rect();
    let screen_width = screen_rect.width();
    let screen_height = screen_rect.height();

    let default_side_panel_width = side_dock.width.unwrap_or((screen_width * 0.10).clamp(200., 1000.));
    // we need a way to calculate the minimum size the bottom panel can be so if we change it in the future it wont start crashing again
    let max_side_panel_width = screen_width - 270.; // 270 is the minimum size to fit bottom panel it will crash if smaller than this
    let default_bottom_panel_height = bottom_dock.height.unwrap_or((screen_height * 0.05).clamp(100., 400.));

    let space = get_interface_config_float("ui.spacing");
    egui::TopBottomPanel::top("tool_panel")
        .resizable(false)
        .show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                top_bar_ui(
                    &mut side_dock,
                    &mut bottom_dock,
                    ui,
                    &mut events,
                    &user_input,
                    &editor_state,
                    &mut commands,
                    &camera_options,
                    viewport_camera_state.as_ref(),
                );
            });
        });

    let side_panel_position = editor_state.config.dock.side_panel_position;
    let panel_response = match side_panel_position {
        SidePanelPosition::Left => {
            egui::SidePanel::left("left_dock_panel")
                .resizable(true)
                .default_width(default_side_panel_width)
                .width_range(250.0..=max_side_panel_width)
                .show(ctx, |ui| {
                    DockArea::new(&mut side_dock.dock_state)
                        .id(egui::Id::new("left_dock_area"))
                        .show_inside(ui, &mut SideTabViewer);
                })
        }
        SidePanelPosition::Right => {
            egui::SidePanel::right("right_dock_panel")
                .resizable(true)
                .default_width(default_side_panel_width)
                .width_range(250.0..=max_side_panel_width)
                .show(ctx, |ui| {
                    DockArea::new(&mut side_dock.dock_state)
                        .id(egui::Id::new("right_dock_area"))
                        .show_inside(ui, &mut SideTabViewer);
                })
        }
    };

    let new_width = panel_response.response.rect.width();
    if new_width != default_side_panel_width {
        side_dock.width = Some(new_width);
    }

    let bottom_response = egui::TopBottomPanel::bottom("bottom_dock_panel")
        .resizable(true)
        .default_height(default_bottom_panel_height)
        .height_range(150.0..=(screen_height * 0.9))
        .show(ctx, |ui| {
            ui.add_space(space);
            DockArea::new(&mut bottom_dock.dock_state)
                .id(egui::Id::new("bottom_dock_area"))
                .show_inside(ui, &mut BottomTabViewer);
        });

    let new_height = bottom_response.response.rect.height();
    if new_height != default_bottom_panel_height {
        bottom_dock.height = Some(new_height);
    }
}
