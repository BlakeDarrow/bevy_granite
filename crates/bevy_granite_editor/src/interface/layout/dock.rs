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
    camera::Camera,
    ecs::{
        query::With,
        system::{Commands, Query, Single},
    },
    prelude::{Res, ResMut},
    core_pipeline::core_3d::Camera3d,
    ecs::system::Commands,
    prelude::{Entity, Name, Res, ResMut, With, Without},
    render::camera::RenderTarget,
};
use bevy_egui::{egui, EguiContexts};
use bevy_granite_core::{UICamera, UserInput};
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
    mut view_port: Query<&mut Camera, With<crate::ViewPortCamera>>,
    camera_query: Query<
        (Entity, Option<&Name>, &Camera),
        (With<Camera3d>, Without<UICamera>, Without<EditorViewportCamera>),
    >, // some changes from #78
    viewport_camera_state: Res<ViewportCameraState>,
) {
    let mut camera_options: Vec<(Entity, String)> = camera_query
        .iter()
        .filter(|(_, _, camera)| matches!(camera.target, RenderTarget::Window(_)))
        .map(|(entity, name, _)| {
            let label = name
                .map(|n| n.as_str().to_string())
                .unwrap_or_else(|| format!("Camera {}", entity.index()));
            (entity, label)
        })
        .collect();
    camera_options.sort_by(|a, b| a.1.cmp(&b.1));

    let ctx = contexts.ctx_mut().expect("Egui context to exist");
    let screen_rect = ctx.screen_rect();
    let screen_width = screen_rect.width();
    let screen_height = screen_rect.height();

    let right_panel_width = (screen_width * 0.10).clamp(200., 1000.);
    let bottom_panel_height = (screen_height * 0.05).clamp(100., 400.);

    let space = get_interface_config_float("ui.spacing");
    let mut left = false;
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
    match side_panel_position {
        SidePanelPosition::Left => {
            left = true;
            egui::SidePanel::left("left_dock_panel")
                .resizable(true)
                .default_width(right_panel_width)
                .width_range(250.0..=(screen_width * 0.9))
                .show(ctx, |ui| {
                    DockArea::new(&mut side_dock.dock_state)
                        .id(egui::Id::new("left_dock_area"))
                        .show_inside(ui, &mut SideTabViewer);
                });
        }
        SidePanelPosition::Right => {
            egui::SidePanel::right("right_dock_panel")
                .resizable(true)
                .default_width(right_panel_width)
                .width_range(250.0..=(screen_width * 0.9))
                .show(ctx, |ui| {
                    DockArea::new(&mut side_dock.dock_state)
                        .id(egui::Id::new("right_dock_area"))
                        .show_inside(ui, &mut SideTabViewer);
                });
        }
    }

    egui::TopBottomPanel::bottom("bottom_dock_panel")
        .resizable(true)
        .default_height(bottom_panel_height)
        .height_range(150.0..=(screen_height * 0.9))
        .show(ctx, |ui| {
            ui.add_space(space);
            DockArea::new(&mut bottom_dock.dock_state)
                .id(egui::Id::new("bottom_dock_area"))
                .show_inside(ui, &mut BottomTabViewer);
        });

    let size = ctx.available_rect();
    for mut camera in view_port.iter_mut() {
        let width = (size.width() * 1.5) as u32;
        let height = (size.height() * 1.5) as u32;
        let Some(viewport) = camera.viewport.as_mut() else {
            continue;
        };
        if left {
            viewport.physical_position.x = screen_width as u32 - width;
        } else {
            viewport.physical_position.x = 0;
        }
        viewport.physical_position.y = (size.min.y * 1.5) as u32;
        viewport.physical_size = bevy::prelude::UVec2::new(width, height);
    }
}
