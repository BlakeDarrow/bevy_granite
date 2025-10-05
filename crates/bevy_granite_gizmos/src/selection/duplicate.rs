use super::{RequestDuplicateAllSelectionEvent, RequestDuplicateEntityEvent};
use crate::{gizmos::GizmoChildren, selection::Selected};
use bevy::{
    asset::Assets,
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query},
    },
    mesh::{Mesh, Mesh3d},
    prelude::{AppTypeRegistry, ChildOf, Children, MessageReader, ReflectComponent, Res, World},
};
use bevy_granite_core::{
    entities::GraniteType, EditorIgnore, HasRuntimeData, IconProxy, IdentityData,
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};
use uuid::Uuid;

pub fn duplicate_entity_system(
    mut commands: Commands,
    mut duplicate_event_reader: MessageReader<RequestDuplicateEntityEvent>,
    type_registry: Res<AppTypeRegistry>,
) {
    for event in duplicate_event_reader.read() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Duplicate Event"
        );
        let to_duplicate = event.entity;
        let registry = type_registry.clone();

        commands.queue(move |world: &mut World| {
            let original_parent = world
                .get_entity(to_duplicate)
                .ok()
                .and_then(|entity_ref| entity_ref.get::<ChildOf>())
                .map(|parent| parent.parent());

            duplicate_entity_recursive(world, to_duplicate, original_parent, &registry);
        });
    }
}

pub fn duplicate_all_selection_system(
    mut commands: Commands,
    mut duplicate_event_reader: MessageReader<RequestDuplicateAllSelectionEvent>,
    type_registry: Res<AppTypeRegistry>,
    selected: Query<Entity, With<Selected>>,
) {
    for _event in duplicate_event_reader.read() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Duplicate All Selection Event"
        );
        let registry = type_registry.clone();
        let entities_to_duplicate: Vec<Entity> = selected.iter().collect();
        commands.queue(move |world: &mut World| {
            for entity in entities_to_duplicate {
                let original_parent = world
                    .get_entity(entity)
                    .ok()
                    .and_then(|entity_ref| entity_ref.get::<ChildOf>())
                    .map(|parent| parent.parent());

                duplicate_entity_recursive(world, entity, original_parent, &registry);
            }
        });
    }
}

fn duplicate_entity_recursive(
    world: &mut World,
    entity_to_duplicate: Entity,
    new_parent: Option<Entity>,
    registry: &AppTypeRegistry,
) -> Option<Entity> {
    let entity_info = collect_entity_info(world, entity_to_duplicate)?;
    let new_entity = create_new_entity(world, new_parent);

    // Handle unique mesh cloning BEFORE copying other components
    let needs_unique = world
        .get::<IdentityData>(entity_to_duplicate)
        .map(|identity| identity.class.needs_unique_handle())
        .unwrap_or(false);

    if needs_unique {
        // Handle mesh cloning - for entities that need unique handles
        if let Some(mesh_handle) = world.get::<Mesh3d>(entity_to_duplicate).cloned() {
            if let Some(mut mesh_assets) = world.get_resource_mut::<Assets<Mesh>>() {
                if let Some(original_mesh) = mesh_assets.get(&mesh_handle) {
                    let cloned_mesh = original_mesh.clone();
                    let new_handle = mesh_assets.add(cloned_mesh);

                    // Add the new handle to the new entity
                    if let Ok(mut entity_mut) = world.get_entity_mut(new_entity) {
                        entity_mut.insert(Mesh3d(new_handle));
                    }
                }
            }
        }
    }

    copy_components_safe(
        world,
        entity_to_duplicate,
        new_entity,
        &entity_info.component_type_ids,
        registry,
    );

    log_copied_components(world, new_entity);

    for child_entity in entity_info.children {
        duplicate_entity_recursive(world, child_entity, Some(new_entity), registry);
    }

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Blank,
        "----------"
    );

    log!(
        LogType::Editor,
        LogLevel::OK,
        LogCategory::Entity,
        "Successfully duplicated entity",
    );

    Some(new_entity)
}

struct EntityInfo {
    component_type_ids: Vec<std::any::TypeId>,
    children: Vec<Entity>,
}

fn collect_entity_info(world: &World, entity: Entity) -> Option<EntityInfo> {
    let entity_ref = world.get_entity(entity).ok()?;

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Blank,
        "----------"
    );
    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Entity,
        "Components to copy:"
    );

    // Check if this entity needs unique handles
    let component_type_ids: Vec<std::any::TypeId> = entity_ref
        .archetype()
        .components()
        .iter()
        .filter_map(|component_id| {
            let component_info = world.components().get_info(component_id.clone())?;

            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Entity,
                "\t{}",
                component_info.name().to_string()
            );

            component_info.type_id()
        })
        .collect();

    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Blank,
        "----------"
    );

    let children: Vec<Entity> = entity_ref
        .get::<Children>()
        .map(|children| {
            children
                .iter()
                .copied()
                .filter(|&child| {
                    world
                        .get_entity(child)
                        // Do NOT include GizmoChildren, IconProxy, or any gizmo entities in duplication
                        .map(|entity_ref| {
                            !entity_ref.contains::<GizmoChildren>()
                                && !entity_ref.contains::<IconProxy>()
                                && !entity_ref
                                    .get::<EditorIgnore>()
                                    .map(|ignore| ignore.contains(EditorIgnore::GIZMO))
                                    .unwrap_or(false)
                        })
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();

    Some(EntityInfo {
        component_type_ids,
        children,
    })
}

fn create_new_entity(world: &mut World, new_parent: Option<Entity>) -> Entity {
    let mut entity_builder = world.spawn_empty();
    let new_entity = entity_builder.insert(HasRuntimeData).id();

    if let Some(parent) = new_parent {
        entity_builder.insert(ChildOf(parent));
    }

    new_entity
}

fn copy_components_safe(
    world: &mut World,
    source_entity: Entity,
    target_entity: Entity,
    component_type_ids: &[std::any::TypeId],
    registry: &AppTypeRegistry,
) {
    let registry_guard = registry.read();

    let needs_unique = world
        .get::<IdentityData>(source_entity)
        .map(|identity| identity.class.needs_unique_handle())
        .unwrap_or(false);

    let mut skip_components = vec![
        std::any::TypeId::of::<ChildOf>(),
        std::any::TypeId::of::<Children>(),
    ];

    // Things like rectangle brushes need unique handles, as we directly edit the vert data in editor
    if needs_unique {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "Requesting unique handle"
        );
        skip_components.push(std::any::TypeId::of::<Mesh3d>());
    }

    for &type_id in component_type_ids {
        if skip_components.contains(&type_id) {
            continue;
        }

        let type_registration = match registry_guard.get(type_id) {
            Some(reg) => reg,
            None => {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Component with TypeId {:?} is not registered for reflection, skipping.",
                    type_id
                );
                continue;
            }
        };

        let reflect_component = match type_registration.data::<ReflectComponent>() {
            Some(rc) => rc,
            None => {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Component with TypeId {:?} does not support ReflectComponent, but registered. Attempting alternative copy.",
                    type_id
                );
                continue;
            }
        };

        let source_ref = match world.get_entity(source_entity) {
            Ok(er) => er,
            Err(e) => {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::Entity,
                    "Error: {:?} Source entity {:?} does not exist, skipping component {:?}.",
                    e,
                    source_entity,
                    type_id
                );
                continue;
            }
        };

        let reflected_component = match reflect_component.reflect(source_ref) {
            Some(rc) => rc,
            None => {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Entity,
                    "Component with TypeId {:?} could not be reflected, skipping.",
                    type_id
                );
                continue;
            }
        };

        // Special handling for IdentityData to generate a new UUID
        if type_id == std::any::TypeId::of::<IdentityData>() {
            if let Some(source_identity) = world.get::<IdentityData>(source_entity) {
                let mut new_identity = source_identity.clone();
                new_identity.uuid = Uuid::new_v4(); // Generate new UUID for the duplicate

                if let Ok(mut target_ref) = world.get_entity_mut(target_entity) {
                    target_ref.insert(new_identity);
                }
            }
        } else {
            if let Ok(cloned_component) = reflected_component.reflect_clone() {
                if let Ok(mut target_ref) = world.get_entity_mut(target_entity) {
                    reflect_component.insert(&mut target_ref, &*cloned_component, &registry_guard);
                }
            }
        }
    }
}

fn log_copied_components(world: &World, entity: Entity) {
    let mut component_names = Vec::new();
    if let Ok(entity_ref) = world.get_entity(entity) {
        for archetype_component_id in entity_ref.archetype().components() {
            if let Some(component_info) =
                world.components().get_info(archetype_component_id.clone())
            {
                component_names.push(component_info.name());
            }
        }
    }

    log!(
        LogType::Editor,
        LogLevel::OK,
        LogCategory::Entity,
        "Components copied"
    );
    for component in component_names {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Entity,
            "\t{}",
            component
        );
    }
}
