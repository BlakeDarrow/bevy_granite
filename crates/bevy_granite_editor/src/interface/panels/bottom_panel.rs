use bevy::prelude::Resource;
use bevy_egui::egui;
use egui_dock::{DockState, NodeIndex, TabViewer};
use serde::{Deserialize, Serialize};

use crate::interface::tabs::{
    debug_tab_ui, events_tab_ui, log_tab_ui, DebugTabData, EventsTabData, LogTabData,
};

#[derive(Resource, Clone)]
pub struct BottomDockState {
    pub dock_state: DockState<BottomTab>,
    pub height: Option<f32>,
}

impl Default for BottomDockState {
    fn default() -> Self {
        let log_tab = BottomTab::Log {
            data: LogTabData::default(),
        };
        let debug_tab = BottomTab::Debug {
            data: DebugTabData::default(),
        };
        let events_tab = BottomTab::Events {
            data: EventsTabData::default(),
        };

        let mut dock_state = DockState::new(vec![debug_tab]);

        let surface = dock_state.main_surface_mut();

        let [_debug_node, remaining] =
            surface.split_right(NodeIndex::root(), 0.33, vec![events_tab]);
        let [_events_node, _log_node] = surface.split_right(remaining, 0.5, vec![log_tab]);

        Self { dock_state, height: None }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum BottomTab {
    Log {
        #[serde(skip)]
        data: LogTabData,
    },
    Debug {
        #[serde(skip)]
        data: DebugTabData,
    },
    Events {
        #[serde(skip)]
        data: EventsTabData,
    },
}

#[derive(Resource)]
pub struct BottomTabViewer;

impl TabViewer for BottomTabViewer {
    type Tab = BottomTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            BottomTab::Log { data, .. } => log_tab_ui(ui, data),
            BottomTab::Debug { data, .. } => debug_tab_ui(ui, data),
            BottomTab::Events { data, .. } => events_tab_ui(ui, data),
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            BottomTab::Log { .. } => "Log".into(),
            BottomTab::Debug { .. } => "Debug".into(),
            BottomTab::Events { .. } => "Events".into(),
        }
    }
}
