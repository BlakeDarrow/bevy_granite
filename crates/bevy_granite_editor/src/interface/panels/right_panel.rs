use bevy::prelude::Resource;
use bevy_egui::egui;
use egui::{Ui, WidgetText};
use egui_dock::{DockState, NodeIndex, TabViewer};
use serde::{Deserialize, Serialize};

use crate::interface::{
    tabs::{
        editor_settings::ui::editor_settings_tab_ui, entity_editor::tab::entity_editor_tab_ui,
        node_tree::node_tree_tab_ui, EditorSettingsTabData, EntityEditorTabData, NodeTreeTabData
    },
};

#[derive(Resource, Clone)]
pub struct SideDockState {
    pub dock_state: DockState<SideTab>,
    pub width: Option<f32>,
}

impl Default for SideDockState {
    fn default() -> Self {
        let node_tree_tab = SideTab::NodeTree {
            data: Box::new(NodeTreeTabData::default()),
        };
        let editor_settings_tab = SideTab::EditorSettings {
            data: Box::new(EditorSettingsTabData::default()),
        };
        let entity_editor_tab = SideTab::EntityEditor {
            data: Box::new(EntityEditorTabData::default()),
        };

        let mut dock_state = DockState::new(vec![node_tree_tab, editor_settings_tab]);
        let surface = dock_state.main_surface_mut();
        let [_old_node, _entity_editor_node] =
            surface.split_below(NodeIndex::root(), 0.3, vec![entity_editor_tab]);

        Self { dock_state, width: None }
    }
}

#[derive(PartialEq)]
pub enum SideTabType {
    EntityEditor,
    NodeTree,
    EditorSettings,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SideTab {
    EntityEditor {
        #[serde(skip)]
        data: Box<EntityEditorTabData>,
    },
    NodeTree {
        #[serde(skip)]
        data: Box<NodeTreeTabData>,
    },
    EditorSettings {
        #[serde(skip)]
        data: Box<EditorSettingsTabData>,
    },
}

impl SideTab {
    pub fn get_type(&self) -> SideTabType {
        match self {
            SideTab::EntityEditor { .. } => SideTabType::EntityEditor,
            SideTab::NodeTree { .. } => SideTabType::NodeTree,
            SideTab::EditorSettings { .. } => SideTabType::EditorSettings,
        }
    }

    pub fn default_from_type(tab_type: SideTabType) -> Self {
        match tab_type {
            SideTabType::EntityEditor => SideTab::EntityEditor {
                data: Box::default(),
            },
            SideTabType::NodeTree => SideTab::NodeTree {
                data: Box::default(),
            },
            SideTabType::EditorSettings => SideTab::EditorSettings {
                data: Box::default(),
            },
        }
    }
}

#[derive(Resource)]
pub struct SideTabViewer;

impl TabViewer for SideTabViewer {
    type Tab = SideTab;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            SideTab::NodeTree { data } => {
                node_tree_tab_ui(ui, data);
            }
            SideTab::EditorSettings { data } => {
                editor_settings_tab_ui(ui, data);
            }
            SideTab::EntityEditor { data } => {
                entity_editor_tab_ui(ui, data);
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            SideTab::NodeTree { .. } => "Entities".into(),
            SideTab::EditorSettings { .. } => "Settings".into(),
            SideTab::EntityEditor { .. } => "Entity Editor".into(),
        }
    }
}
