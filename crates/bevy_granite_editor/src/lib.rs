use crate::setup::{editor_info, setup_ui_style};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

pub mod editor_state;
pub mod entities;
pub mod input;
pub mod interface;
pub mod setup;
pub mod utils;
pub mod viewport;

use editor_state::ConfigPlugin;
use entities::AssetPlugin;
use input::InputPlugin;
use interface::InterfacePlugin;
use viewport::ViewportPlugin;

pub use editor_state::{
    get_interface_config_float, get_interface_config_str, update_editor_config_field, HELP_CONFIG,
    UI_CONFIG,
};
pub use entities::get_entity_bounds_or_fallback;
pub use interface::events::{
    RequestCameraEntityFrame, RequestEditorToggle, RequestNewParent, RequestRemoveChildren,
    RequestRemoveParents, RequestToggleCameraSync,
};

pub struct BevyGraniteEditor {
    pub active: bool,
    pub default_world: String,
}

impl Plugin for BevyGraniteEditor {
    fn build(&self, app: &mut App) {
        app
            //
            //Plugins
            //
            .add_plugins(FrameTimeDiagnosticsPlugin::default()) // Bevy internal frame plugin
            //
            // Internal plugins
            .add_plugins(InputPlugin)
            .add_plugins(InterfacePlugin)
            .add_plugins(ViewportPlugin) // Required
            .add_plugins(AssetPlugin) // Required
            .add_plugins(ConfigPlugin {
                editor_active: self.active,
                default_world: self.default_world.clone(),
            }) // Required
            //
            // Startup
            //
            .add_systems(PostStartup, (setup_ui_style, editor_info));

        app.add_plugins(bevy_granite_expose::BevyGraniteExposePlugin); // this will register internal bevy components so they can be used in the editor
    }
}

/// Marker component for the camera used as the editor viewport
/// IDK whats with egui it seems to eat the normal view of the camera
#[derive(Component)]
pub struct ViewPortCamera;
