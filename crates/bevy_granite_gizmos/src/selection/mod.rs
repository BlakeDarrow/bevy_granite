use bevy::ecs::{
    component::Component, lifecycle::HookContext, message::Messages, world::DeferredWorld,
};

pub mod duplicate;
pub mod events;
pub mod manager;
pub mod plugin;
pub mod ray;

/// Just the active selection marker
#[derive(Component)]
#[require(Selected)]
#[component(on_add = ActiveSelection::on_add, on_remove = ActiveSelection::on_remove)]
pub struct ActiveSelection;

impl ActiveSelection {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.write_message(SpawnGizmoEvent(ctx.entity));
    }
    fn on_remove(mut world: DeferredWorld, ctx: HookContext) {
        if let Ok(_entity_commands) = world.commands().get_entity(ctx.entity) {
            if let Some(gizmos) = world.entity(ctx.entity).get::<crate::gizmos::Gizmos>() {
                let targets: Vec<_> = gizmos.entities().iter().copied().collect();
                let mut commands = world.commands();
                for target in targets {
                    // Use try_despawn to avoid errors when entity doesn't exist
                    if let Ok(mut entity_commands) = commands.get_entity(target) {
                        entity_commands.try_despawn();
                    }
                }
            }
        }
    }
}

/// ALL selection including the active selection marker
#[derive(Component, Default)]
pub struct Selected;

pub use duplicate::{duplicate_all_selection_system, duplicate_entity_system};
pub use events::{EntityEvents, RequestDuplicateAllSelectionEvent, RequestDuplicateEntityEvent};
pub use manager::{apply_pending_parents, handle_picking_selection, select_entity};
pub use plugin::SelectionPlugin;
pub use ray::{RaycastCursorLast, RaycastCursorPos};

use crate::gizmos::SpawnGizmoEvent;
