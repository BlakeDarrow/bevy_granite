use super::{
    apply_pending_parents, duplicate_all_selection_system, duplicate_entity_system,
    handle_picking_selection, select_entity, RaycastCursorLast, RaycastCursorPos,
    RequestDuplicateAllSelectionEvent, RequestDuplicateEntityEvent,
};
use crate::{is_gizmos_active, selection::manager::deselect_entity};
use bevy::{
    app::{App, Plugin, PostUpdate, Update},
    ecs::schedule::IntoScheduleConfigs,
    math::Vec3,
};

pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Events
            //
            .add_message::<RequestDuplicateEntityEvent>()
            .add_message::<RequestDuplicateAllSelectionEvent>()
            //
            // Resources
            //
            .insert_resource(RaycastCursorLast {
                position: Vec3::ZERO,
            })
            .insert_resource(RaycastCursorPos {
                position: Vec3::ZERO,
            })
            //
            // Events
            //
            //
            // Schedule system
            //
            .add_systems(
                Update,
                (
                    duplicate_entity_system.after(handle_picking_selection),
                    duplicate_all_selection_system.after(handle_picking_selection),
                )
                    .run_if(is_gizmos_active),
            )
            .add_systems(PostUpdate, (apply_pending_parents).run_if(is_gizmos_active))
            .add_observer(handle_picking_selection)
            .add_observer(super::manager::single_active)
            .add_observer(select_entity)
            .add_observer(deselect_entity);
    }
}
