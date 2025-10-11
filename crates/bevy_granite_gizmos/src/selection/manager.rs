use crate::selection::{events::EntityEvents, ActiveSelection, Selected};
use bevy::{
    ecs::{lifecycle::Add, observer::On},
    prelude::{Component, Entity, Query, Res, With},
};
use bevy::{
    ecs::{query::QueryEntityError, system::Commands},
    picking::events::{Click, Pointer},
};
use bevy_granite_core::{EditorIgnore, IconProxy, UserInput};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

pub fn apply_pending_parents(mut commands: Commands, query: Query<(Entity, &ParentTo)>) {
    for (entity, parent_to) in &query {
        if let Ok(mut parent) = commands.get_entity(parent_to.0) {
            parent.add_children(&[entity]);
            commands.entity(entity).remove::<ParentTo>();
        } else {
            log!(
                LogType::Editor,
                LogLevel::Critical,
                LogCategory::Entity,
                "Failed to parent entity {:?} to {:?}",
                entity,
                parent_to.0
            );
        }
    }
}

#[derive(Component)]
pub struct ParentTo(pub Entity);

// this is incharge of setting entities into the selected state
pub fn select_entity(
    event: On<EntityEvents>,
    mut commands: Commands,
    current: Query<Entity, With<Selected>>,
    active_selection: Query<(), With<ActiveSelection>>,
) {
    let (first, add, others) = match event.event() {
        EntityEvents::Select { target, additive } => (*target, *additive, None),
        EntityEvents::SelectRange { range, additive } => {
            let Some(first) = range.first() else {
                log! {
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::Entity,
                    "Failed to select range: no entities found"
                };
                return;
            };
            (*first, *additive, Some(&range[1..]))
        }
        _ => {
            return;
        }
    };
    if !add {
        for entity in current.iter() {
            commands.entity(entity).remove::<Selected>();
        }
    }
    if active_selection.get(first).is_ok() {
        commands
            .entity(first)
            .remove::<(ActiveSelection, Selected)>();
        if others.is_none() {
            return;
        }
    } else {
        commands.entity(first).insert(ActiveSelection);
    }
    if let Some(rest) = others {
        for entity in rest {
            commands.entity(*entity).insert(Selected);
        }
    }
}

// Used when we get a single entity deselected
pub fn deselect_entity(
    event: On<EntityEvents>,
    mut commands: Commands,
    selection: Query<Entity, With<Selected>>,
) {
    match event.event() {
        EntityEvents::Deselect { target } => {
            commands
                .entity(*target)
                .remove::<(ActiveSelection, Selected)>();
        }
        EntityEvents::DeselectRange { range } => {
            for entity in range {
                commands
                    .entity(*entity)
                    .remove::<(ActiveSelection, Selected)>();
            }
        }
        EntityEvents::DeselectAll => {
            for entity in selection.iter() {
                commands
                    .entity(entity)
                    .remove::<(ActiveSelection, Selected)>();
            }
        }
        _ => {}
    }
}

/// when an entity gets ActiveSelection added to it we check if there is already an entity with ActiveSelection
/// if there is, we remove it for the other entity
pub fn single_active(
    add_active: On<Add, ActiveSelection>,
    active_selection: Query<Entity, With<ActiveSelection>>,
    mut commands: Commands,
) {
    if active_selection.single().is_err() {
        for entity in &active_selection {
            if entity != add_active.entity {
                commands.entity(entity).remove::<ActiveSelection>();
            }
        }
    }
}

pub fn handle_picking_selection(
    mut on_click: On<Pointer<Click>>,
    mut commands: Commands,
    ignored: Query<&EditorIgnore>,
    icon_proxy_query: Query<&IconProxy>,
    user_input: Res<UserInput>,
) {
    if on_click.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }
    match ignored.get(on_click.trigger().original_event_target) {
        Ok(to_ignore) => {
            if to_ignore.contains(EditorIgnore::PICKING) {
                return;
            }
        }
        Err(QueryEntityError::EntityDoesNotExist(_)) => {
            log!("Entity does not exist: {}", on_click.entity.index());
            return;
        }
        Err(_) => {}
    }
    if user_input.mouse_over_egui {
        return;
    }

    if on_click.entity.index() == 0 {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Clicked on empty space, deselecting all entities"
        );
        on_click.propagate(false);
        commands.trigger(EntityEvents::DeselectAll);
        return;
    }

    on_click.propagate(false);
    let mut entity = on_click.entity;

    // redirect to icon target
    if let Ok(icon_proxy) = icon_proxy_query.get(entity) {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Icon proxy clicked, redirecting to target entity {}",
            icon_proxy.target_entity.index()
        );
        entity = icon_proxy.target_entity;
    }

    commands.trigger(EntityEvents::Select {
        target: entity,
        additive: user_input.shift_left.any,
    });
}
