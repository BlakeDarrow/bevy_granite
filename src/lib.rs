//! # BevyGranite Plugin Library
//!
//! This crate provides a unified interface to manage 3d scenes utilizing
//! a custom editor built with egui.
//!
//! ## Features
//!
//! This crate supports conditional compilation through Cargo features:
//!
//! - **`default`**: Enables `core`, `editor`, and `gizmos` features
//! - **`core`**: Essential functionality including logging, macros, and serialization
//! - **`editor`**: In-game editor with UI components and entity management (includes `core` and `gizmos`)
//! - **`gizmos`**: Visual debugging tools and entity manipulation gizmos (includes `core`)
//!
//! ### Feature Components
//!
//! - **Logging**: Advanced logging system with categories and levels (core)
//! - **Editor**: In-game editor with UI components and entity management (editor)
//! - **Macros**: Convenient macros for component registration and granite components (core)
//! - **Core**: Essential serialization and world management functionality (core)
//! - **Gizmos**: Visual debugging tools and entity manipulation gizmos (gizmos)
//!
//! ## Usage
//!
//! ### With Default Features (core + editor)
//!
//! Add the BevyGranite plugin group to your Bevy app:
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_granite::prelude::*;
//! const STARTING_WORLD: &str = "scenes/starting.scene";
//!
//!fn main() {
//!    let mut app = App::new();
//!    register_editor_components!();
//!
//!    app.add_plugins(DefaultPlugins)
//!        .add_plugins(bevy_granite::BevyGranite {
//!            default_world: STARTING_WORLD.to_string(),
//!            ..Default::default()
//!        })
//!        .add_systems(Startup, setup)
//!        .run();
//!}
//! ```
//!
//! ### Core Only (without editor)
//!
//! To use only the core functionality without the editor:
//!
//! ```toml
//! [dependencies]
//! bevy_granite = { version = "0.3.0", default-features = false, features = ["core"] }
//! ```

use bevy::app::{PluginGroup, PluginGroupBuilder};

#[cfg(feature = "core")]
pub use bevy_granite_core;
#[cfg(feature = "editor")]
pub use bevy_granite_editor;
#[cfg(feature = "gizmos")]
pub use bevy_granite_gizmos;
#[cfg(feature = "core")]
pub use bevy_granite_logging;
#[cfg(feature = "core")]
pub use bevy_granite_macros;

/// Initial configuration of BevyGranite.
///
/// This struct allows you to configure essential behavior such as its enabled state and default world for UI.
pub struct BevyGranite {
    /// Whether the editor functionality should be active
    pub active: bool,
    /// String relative path to the default world file for UI
    pub default_world: String,
    /// Whether to enable log setup, essentially controlling the logging system
    pub logging: bool,
}

impl Default for BevyGranite {
    fn default() -> Self {
        Self {
            active: true,
            default_world: "scenes/default.mat".to_string(),
            logging: true,
        }
    }
}

impl PluginGroup for BevyGranite {
    /// Builds the complete BevyGranite plugin group.
    ///
    /// This method assembles all the individual plugins in the correct order utilizing feature sets
    ///
    fn build(self) -> PluginGroupBuilder {
        let mut builder = PluginGroupBuilder::start::<Self>()
            // Required plugins
            .add(bevy_inspector_egui::DefaultInspectorConfigPlugin);

        #[cfg(feature = "core")]
        {
            builder = builder.add(bevy_granite_core::BevyGraniteCore {
                logging: self.logging,
            });
        }

        #[cfg(feature = "gizmos")]
        {
            builder = builder.add(bevy_granite_gizmos::BevyGraniteGizmos);
        }

        #[cfg(feature = "editor")]
        {
            builder = builder.add(bevy_granite_editor::BevyGraniteEditor {
                active: self.active,
                default_world: self.default_world,
            });
        }

        builder
    }
}

/// Prelude module providing convenient access to frequently used items.
///
/// ## Categories
///
/// - **Macros**: Component registration and granite component macros
/// - **Logging**: Log functions and configuration types
/// - **Editor**: UI events, camera management, and entity tree utilities
/// - **Core**: World loading/saving events and serialization utilities
/// - **Gizmos**: Entity selection, duplication, and spawning events
pub mod prelude {
    pub use crate::BevyGranite;

    #[cfg(feature = "core")]
    pub use crate::{
        bevy_granite_core,
        bevy_granite_core::{
            absolute_asset_to_rel, rel_asset_to_absolute, BridgeTag, MainCamera,
            RequestDespawnBySource, RequestDespawnSerializableEntities, RequestLoadEvent,
            RequestReloadEvent, RequestSaveEvent, SaveSettings, SpawnSource, TreeHiddenEntity,
            UICamera, WorldLoadSuccessEvent, WorldSaveSuccessEvent, RequestLoadBatchEvent, WorldLoadBatchSuccessEvent
        },
        bevy_granite_logging::{log, LogCategory, LogLevel, LogType},
        bevy_granite_macros::{granite_component, register_editor_components, ui_callable_events},
    };

    #[cfg(feature = "gizmos")]
    pub use crate::bevy_granite_gizmos::{
        EntityEvents, RequestDuplicateAllSelectionEvent, RequestDuplicateEntityEvent,
    };

    #[cfg(feature = "editor")]
    pub use crate::bevy_granite_editor::{
        RequestCameraEntityFrame, RequestEditorToggle, RequestNewParent, RequestRemoveChildren,
        RequestRemoveParents, RequestToggleCameraSync,
    };

    #[cfg(feature = "editor")]
    pub use crate::bevy_granite_editor::interface::tabs::events::ui::register_ui_callable_events_with_senders;
}
