use super::camera::{
    add_editor_camera, add_gizmo_overlay_camera, add_ui_camera, camera_frame_system,
    camera_sync_toggle_system, enforce_viewport_camera_state, gizmo_layers, grid_layers,
    handle_viewport_camera_override_requests, mouse_button_iter, restore_runtime_camera_state,
    sync_cameras_system, sync_gizmo_camera_state, update_viewport_camera_viewports_system,
    CameraSyncState, CameraTarget, InputState, ViewportCameraState,
};
use super::viewmode::{cleanup_scene_light_system, scene_light_system, SceneLightState};
use crate::{
    setup::is_editor_active,
    viewport::{
        cleanup_icon_entities_system, grid::{spawn_viewport_grid, update_grid_system},
        icons::register_embedded_class_icons, relationship_line_system,
        show_active_selection_bounds_system, show_camera_forward_system,
        show_directional_light_forward_system, show_empty_origin_system,
        show_point_light_range_system, show_selected_entities_bounds_system,
        spawn_icon_entities_system, update_icon_entities_system, DebugRenderer, SelectionRenderer,
    },
};
use bevy::{
    app::{PostUpdate, Startup},
    ecs::schedule::{common_conditions::not, ApplyDeferred, IntoScheduleConfigs}, // from #78
    gizmos::{
        config::{DefaultGizmoConfigGroup, GizmoConfig},
        AppGizmoBuilder,
    },
    prelude::{App, Plugin, Update},
    transform::TransformSystems,
};
use bevy_egui::EguiPrimaryContextPass;

pub struct ViewportPlugin;
impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Resources
            //
            .insert_resource(CameraTarget::default())
            .insert_resource(CameraSyncState::default())
            .insert_resource(InputState::default()) // FIX: Use UserInput
            .insert_resource(ViewportCameraState::default())
            .insert_resource(SceneLightState::default())
            //
            // Debug gizmo groups/config
            //
            .init_gizmo_group::<DefaultGizmoConfigGroup>()
            .insert_gizmo_config(
                DefaultGizmoConfigGroup,
                GizmoConfig {
                    render_layers: grid_layers(),
                    ..Default::default()
                },
            )
            .init_gizmo_group::<SelectionRenderer>()
            .insert_gizmo_config(
                SelectionRenderer,
                GizmoConfig {
                    render_layers: gizmo_layers(),
                    ..Default::default()
                },
            )
            .init_gizmo_group::<DebugRenderer>()
            .insert_gizmo_config(
                DebugRenderer,
                GizmoConfig {
                    depth_bias: -1.0,
                    render_layers: gizmo_layers(),
                    ..Default::default()
                },
            )
            //
            // Schedule system
            //
            .add_systems(Startup, register_embedded_class_icons)
            .add_systems(
                Startup,
                (
                    add_editor_camera,
                    add_gizmo_overlay_camera,
                    add_ui_camera,
                    spawn_viewport_grid,
                    ApplyDeferred,
                    bevy_egui::update_ui_size_and_scale_system,
                )
                    .chain(),
            )
            .add_systems(Update, update_grid_system.run_if(is_editor_active))
            .add_systems(Update, mouse_button_iter.run_if(is_editor_active)) // FIX: Use UserInput
            .add_systems(Update, camera_frame_system.run_if(is_editor_active))
            .add_systems(Update, camera_sync_toggle_system.run_if(is_editor_active))
            .add_systems(Update, scene_light_system.run_if(is_editor_active))
            .add_systems(Update, cleanup_scene_light_system.run_if(not(is_editor_active)))
            .add_systems(
                Update,
                (handle_viewport_camera_override_requests, enforce_viewport_camera_state)
                    .chain()
                    .run_if(is_editor_active),
            )
            .add_systems(
                EguiPrimaryContextPass,
                update_viewport_camera_viewports_system.run_if(is_editor_active),
            )
            .add_systems(
                Update,
                restore_runtime_camera_state.run_if(not(is_editor_active)),
            )
            // No run if here because this will hide the gizmos if editor is not active
            .add_systems(Update, update_icon_entities_system)
            .add_systems(
                Update,
                (spawn_icon_entities_system, cleanup_icon_entities_system).run_if(is_editor_active),
            )
            .add_systems(
                // Different gizmo visualizers per type
                PostUpdate,
                (
                    show_directional_light_forward_system,
                    show_camera_forward_system,
                    relationship_line_system,
                    show_point_light_range_system,
                    show_empty_origin_system,
                    show_active_selection_bounds_system,
                    show_selected_entities_bounds_system,
                )
                    .after(TransformSystems::Propagate)
                    .run_if(is_editor_active),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_cameras_system.run_if(is_editor_active),
                    sync_gizmo_camera_state,
                )
                    .chain(),
            );
    }
}
