use crate::interface::{
    cache::entity_cache::{EntityData, EntityUIDataCache},
    tabs::entity_editor::EntityRegisteredData,
};
use bevy::{
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{Entity, Name, Transform, World},
    transform::components::GlobalTransform,
};
use bevy_granite_core::{entities::Unknown, ComponentEditor, IdentityData, TransformData};
use bevy_granite_gizmos::{ActiveSelection, DragState};

pub type EntityCacheQueryItem<'a> = (
    Entity,
    Option<&'a Name>,
    &'a mut Transform,
    &'a mut GlobalTransform,
    Option<&'a mut IdentityData>,
    Option<&'a mut MeshMaterial3d<StandardMaterial>>,
    &'a ActiveSelection,
);

// FIX:
// need new cache?
// WORLD has changes?

// Store all the needed entity data for UI population and manipulation
pub fn update_entity_cache_system(world: &mut World) {
    let mut query = world.query::<EntityCacheQueryItem>();
    let gizmo_drag = world.resource::<DragState>();
    let component_editor = world.resource::<ComponentEditor>();
    let filter = world
        .resource::<crate::interface::SideDockState>()
        .dock_state
        .iter_all_tabs()
        .any(|(_, tab)| match tab {
            crate::interface::panels::right_panel::SideTab::NodeTree { data } => {
                data.filtered_hierarchy
            }
            _ => false,
        });

    let maybe_new_data = if let Some((
        entity,
        name,
        _transform,
        global_transform,
        identity_data,
        material_handle,
        _active,
    )) = query.iter(world).next()
    {
        let new_registered = component_editor.get_reflected_components(world, entity, filter);
        // Use GlobalTransform for UI display (world position), but keep local transform for editing
        let global = global_transform.compute_transform();

        let identity = if let Some(id) = identity_data {
            id.clone()
        } else {
            IdentityData {
                name: name
                    .map(|n| n.to_string())
                    .unwrap_or(format!("Entity {entity:?}")),
                uuid: uuid::Uuid::new_v4(),
                class: bevy_granite_core::GraniteTypes::Unknown(Unknown::default()),
            }
        };

        let new_data = EntityData {
            entity: Some(entity),
            global_transform: TransformData {
                position: global.translation, // World position
                rotation: global.rotation,    // World rotation
                scale: global.scale,          // World scale
            },
            material_handle: material_handle.cloned(),
            identity,
            registered: EntityRegisteredData {
                components: new_registered,
                registered_add_request: None,
                registered_remove_request: None,
                registered_data_changed: false,
            },
            gizmo_drag: gizmo_drag.clone(),
        };
        Some((entity, new_data))
    } else {
        None
    };

    let mut cache = world.resource_mut::<EntityUIDataCache>();

    if let Some((entity, new_data)) = maybe_new_data {
        let is_new_entity = cache.last_entity != Some(entity) || cache.last_entity.is_none();
        if is_new_entity {
            cache.dirty.entity_dirty = true;
        }

        if cache.data.material_handle != new_data.material_handle {
            cache.data.material_handle = new_data.material_handle.clone();
            cache.dirty.material_dirty = true;
        }

        if cache.data.gizmo_drag != new_data.gizmo_drag {
            cache.data.gizmo_drag = new_data.gizmo_drag.clone();
            cache.dirty.gizmo_dirty = true;
        }

        if cache.data.registered.components != new_data.registered.components {
            cache.data.registered = new_data.registered.clone();
            cache.dirty.registered_dirty = true;
        }

        if cache.data.global_transform != new_data.global_transform {
            cache.data.global_transform = new_data.global_transform.clone();
            cache.dirty.global_transform_dirty = true;
        }
        if cache.data.identity != new_data.identity {
            cache.data.identity = new_data.identity.clone();
            cache.dirty.identity_dirty = true;
        }
        if cache.data.entity != new_data.entity {
            cache.data.entity = new_data.entity;
        }
        cache.last_entity = Some(entity);
    } else {
        // Clear all cached data when no entity is selected
        let was_entity_selected = cache.last_entity.is_some();

        if was_entity_selected {
            cache.dirty.entity_dirty = true;
            cache.dirty.global_transform_dirty = true;
            cache.dirty.identity_dirty = true;
            cache.dirty.registered_dirty = true;
            cache.dirty.gizmo_dirty = true;
            cache.dirty.material_dirty = true;
        }

        cache.data.entity = None;
        cache.last_entity = None;
        cache.data.material_handle = None;
        cache.data.global_transform = TransformData::default();
        cache.data.identity = IdentityData::default();
        cache.data.gizmo_drag = DragState::default();
        // Clear registered components when no entity is selected
        cache.data.registered = EntityRegisteredData::default();
    }

    if cache.dirty.entity_dirty || cache.dirty.registered_dirty {
        // Update global ComponentEditor
        if let Some(entity) = cache.data.entity {
            let mut global_component_editor = world.resource_mut::<ComponentEditor>();
            global_component_editor.set_selected_entity(entity);
        }
    }
}
