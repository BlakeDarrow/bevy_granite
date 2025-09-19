use super::{
    cache::update_entity_cache_system,
    events::{
        MaterialDeleteEvent, MaterialHandleUpdateEvent, PopupMenuRequestedEvent,
        RequestCameraEntityFrame, RequestEditorToggle, RequestNewParent, RequestRemoveChildren,
        RequestRemoveParents, RequestToggleCameraSync, SetActiveWorld,
        UserRequestGraniteTypeViaPopup, UserUpdatedComponentsEvent, UserUpdatedIdentityEvent,
        UserUpdatedTransformEvent,
    },
    layout::dock_ui_system,
    popups::{handle_popup_requests_system, show_active_popups_system},
    tabs::{
        handle_material_deletion_system, send_queued_events_system, update_debug_tab_ui_system,
        update_editor_settings_tab_system, update_entity_editor_tab_system,
        update_entity_with_new_components_system, update_entity_with_new_identity_system,
        update_entity_with_new_transform_system, update_log_tab_system,
        update_material_handle_system, update_node_tree_tabs_system, RequestReparentEntityEvent,
    },
    BottomDockState, EntityUIDataCache, PopupState, SideDockState,
};
use crate::{interface::RequestRemoveParentsFromEntities, setup::is_editor_active};
use bevy::{
    app::Update,
    ecs::schedule::IntoScheduleConfigs,
    prelude::{App, Handle, Mesh, Plugin, StandardMaterial},
};
use bevy_egui::EguiPrimaryContextPass;

pub struct InterfacePlugin;
impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Interface events
            //
            .add_event::<MaterialHandleUpdateEvent>()
            .add_event::<MaterialDeleteEvent>()
            .add_event::<UserUpdatedComponentsEvent>()
            .add_event::<UserUpdatedTransformEvent>()
            .add_event::<UserUpdatedIdentityEvent>()
            .add_event::<UserRequestGraniteTypeViaPopup>()
            .add_event::<PopupMenuRequestedEvent>()
            .add_event::<RequestEditorToggle>()
            .add_event::<RequestCameraEntityFrame>()
            .add_event::<RequestToggleCameraSync>()
            .add_event::<RequestNewParent>()
            .add_event::<RequestRemoveChildren>()
            .add_event::<RequestRemoveParents>()
            .add_event::<SetActiveWorld>()
            // need to rework
            .add_event::<RequestReparentEntityEvent>()
            .add_event::<RequestRemoveParentsFromEntities>()
            //
            // Register types
            // If you want to duplicate bevy data you must register the type
            //
            .register_type::<Handle<Mesh>>()
            .register_type::<Handle<StandardMaterial>>()
            //
            // Resources
            //
            .insert_resource(EntityUIDataCache::default())
            .insert_resource(PopupState::default())
            .insert_resource(SideDockState::default())
            .insert_resource(BottomDockState::default())
            //
            // Schedule systems
            //
            .add_systems(
                Update,
                (
                    //
                    // Handle UI requests to update entities
                    //
                    update_entity_with_new_components_system,
                    update_entity_with_new_transform_system,
                    update_entity_with_new_identity_system,
                    //
                    // Actual entity updates from UI changes
                    //
                    update_entity_cache_system,
                    update_material_handle_system,
                    handle_material_deletion_system,
                    //
                    // Layout and Popups
                    //
                    handle_popup_requests_system,
                    //
                    // Interface tabs UI
                    //
                    update_node_tree_tabs_system,
                    update_entity_editor_tab_system,
                    update_editor_settings_tab_system,
                    update_log_tab_system,
                    update_debug_tab_ui_system,
                    update_node_tree_tabs_system,
                )
                    .chain()
                    .run_if(is_editor_active),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (show_active_popups_system, dock_ui_system).run_if(is_editor_active),
            )
            .add_systems(Update, send_queued_events_system.run_if(is_editor_active));
    }
}
