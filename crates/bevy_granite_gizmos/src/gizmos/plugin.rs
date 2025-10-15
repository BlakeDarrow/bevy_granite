use super::register_embedded_rotate_gizmo_mesh;
use super::{
    gizmo_changed_watcher, gizmo_events, handle_init_rotate_drag, handle_rotate_input,
    handle_rotate_reset, scale_gizmo_by_camera_distance_system, DespawnGizmoEvent, GizmoSnap,
    GizmoType, LastSelectedGizmo, NewGizmoConfig, PreviousTransformGizmo, RotateDraggingEvent,
    RotateInitDragEvent, RotateResetDragEvent, SpawnGizmoEvent, TransformDraggingEvent,
    TransformInitDragEvent, TransformResetDragEvent,
    update_rotate_gizmo_rotation_for_mode, update_transform_gizmo_rotation_for_mode,
};
use crate::gizmos::transform::{
    apply_transformations, TransitionDelta,
};
use crate::gizmos::{GizmoMode, NewGizmoType};
use crate::is_gizmos_active;
use bevy::ecs::schedule::common_conditions::any_with_component;
use bevy::{
    app::{App, Plugin, PostUpdate, Startup, Update},
    ecs::schedule::IntoScheduleConfigs,
};

pub struct GizmoPlugin;
impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Resources
            //
            .insert_resource(PreviousTransformGizmo::default())
            .insert_resource(LastSelectedGizmo {
                value: GizmoType::default(),
            })
            .insert_resource(NewGizmoConfig {
                speed_scale: 1.,
                distance_scale: 1.,
                mode: GizmoMode::Global,
            })
            .insert_resource(NewGizmoType(GizmoType::Pointer))
            .insert_resource(GizmoSnap {
                transform_value: 0.,
                rotate_value: 0.,
            })
            .insert_resource(super::transform::drag::TransformDuplicationState::default())
            //
            // Events
            //
            .add_message::<RotateInitDragEvent>()
            .add_message::<RotateDraggingEvent>()
            .add_message::<RotateResetDragEvent>()
            .add_message::<TransformInitDragEvent>()
            .add_message::<TransformDraggingEvent>()
            .add_message::<TransformResetDragEvent>()
            .add_message::<SpawnGizmoEvent>()
            .add_message::<DespawnGizmoEvent>()
            //
            // Schedule system
            //
            .add_systems(Startup, register_embedded_rotate_gizmo_mesh)
            .add_systems(
                Update,
                (
                    gizmo_changed_watcher,
                    gizmo_events,
                    update_transform_gizmo_rotation_for_mode,
                    update_rotate_gizmo_rotation_for_mode,
                    apply_transformations.run_if(any_with_component::<TransitionDelta>),
                )
                    .run_if(is_gizmos_active),
            )
            .add_systems(
                Update,
                (
                    // Rotate gizmo
                    handle_rotate_input,
                    handle_init_rotate_drag.after(handle_rotate_input),
                    handle_rotate_reset.after(handle_rotate_input),
                )
                    .run_if(is_gizmos_active),
            )
            .add_systems(
                PostUpdate,
                (
                    scale_gizmo_by_camera_distance_system.run_if(is_gizmos_active),
                    super::transform::cleanup_axis_line,
                ),
            );
        app.add_observer(super::transform::draw_axis_lines);
    }
}
