use super::camera::{
    add_ui_camera, camera_frame_system, camera_sync_toggle_system, mouse_button_iter,
    sync_cameras_system, CameraSyncState, CameraTarget, InputState,
};
use crate::{
    setup::is_editor_active,
    viewport::{
        cleanup_icon_entities_system, icons::register_embedded_class_icons,
        relationship_line_system, show_active_selection_bounds_system, show_camera_forward_system,
        show_directional_light_forward_system, show_empty_origin_system,
        show_point_light_range_system, show_selected_entities_bounds_system,
        spawn_icon_entities_system, update_grid_system, update_icon_entities_system, DebugRenderer,
        SelectionRenderer,
    },
};
use bevy::{
    app::{PostUpdate, Startup},
    camera::visibility::RenderLayers,
    ecs::schedule::{ApplyDeferred, IntoScheduleConfigs},
    gizmos::{
        config::{DefaultGizmoConfigGroup, GizmoConfig},
        AppGizmoBuilder,
    },
    prelude::{App, Plugin, Update},
    transform::TransformSystems,
};

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
            //
            // Debug gizmo groups/config
            //
            .init_gizmo_group::<DefaultGizmoConfigGroup>()
            .init_gizmo_group::<SelectionRenderer>()
            .insert_gizmo_config(
                SelectionRenderer,
                GizmoConfig {
                    render_layers: RenderLayers::from_layers(&[14]), // 14 is our UI/Gizmo layer.
                    ..Default::default()
                },
            )
            .init_gizmo_group::<DebugRenderer>()
            .insert_gizmo_config(
                DebugRenderer,
                GizmoConfig {
                    depth_bias: -1.0,
                    render_layers: RenderLayers::from_layers(&[14]), // 14 is our UI/Gizmo layer.
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
                    add_ui_camera,
                    ApplyDeferred,
                    bevy_egui::update_ui_size_and_scale_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    update_grid_system,
                    mouse_button_iter, // FIX: Use UserInput
                    camera_frame_system,
                    camera_sync_toggle_system,
                )
                    .run_if(is_editor_active),
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
            .add_systems(PostUpdate, sync_cameras_system.run_if(is_editor_active));
    }
}
