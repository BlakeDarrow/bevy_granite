use super::data::{HierarchyEntry, NodeTreeTabData};
use bevy::prelude::{ChildOf, Entity, Name};
use bevy_granite_core::{GraniteType, IdentityData, SaveSettings, SpawnSource};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};
use std::collections::HashMap;

/// Detects changes in the hierarchy that require UI refresh
pub fn detect_changes<'a>(
    hierarchy_query: impl Iterator<
            Item = (
                Entity,
                &'a Name,
                Option<&'a ChildOf>,
                Option<&'a IdentityData>,
                Option<&'a SpawnSource>,
            ),
        > + Clone,
    changed_hierarchy: bool,
    data: &NodeTreeTabData,
) -> (bool, bool, bool) {
    use std::collections::HashSet;

    let current_entities: HashSet<Entity> =
        hierarchy_query.clone().map(|(e, _, _, _, _)| e).collect();
    let existing_entities: HashSet<Entity> =
        data.hierarchy.iter().map(|entry| entry.entity).collect();

    let entities_changed = current_entities != existing_entities;
    let data_changed = changed_hierarchy;

    let hierarchy_changed = if !entities_changed {
        hierarchy_query
            .into_iter()
            .any(|(entity, _, relation, _, _)| {
                if let Some(entry) = data.hierarchy.iter().find(|e| e.entity == entity) {
                    let current_parent = relation.map(|p| p.parent());
                    entry.parent != current_parent
                } else {
                    true
                }
            })
    } else {
        false
    };

    (entities_changed, data_changed, hierarchy_changed)
}

/// Updates the hierarchy data from ECS query results
pub fn update_hierarchy_data<'a>(
    data: &mut NodeTreeTabData,
    hierarchy_query: impl IntoIterator<
        Item = (
            Entity,
            &'a Name,
            Option<&'a ChildOf>,
            Option<&'a IdentityData>,
            Option<&'a SpawnSource>,
        ),
    >,
    hierarchy_changed: bool,
) {
    if hierarchy_changed {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::UI,
            "Hierarchy relationships changed - refreshing node tree"
        );
    }

    let existing_expanded: HashMap<Entity, bool> = data
        .hierarchy
        .iter()
        .map(|entry| (entry.entity, entry.is_expanded))
        .collect();

    // First, collect all entities and group those with ANY SpawnSource
    let mut real_entities: Vec<HierarchyEntry> = Vec::new();
    let mut file_groups: HashMap<String, Vec<Entity>> = HashMap::new();

    for (entity, name, relation, identity, spawn_source) in hierarchy_query {
        let is_preserve_disk = spawn_source.map_or(false, |source| {
            matches!(source.save_settings_ref(), SaveSettings::PreserveDiskFull)
        });
        let is_preserve_disk_transform = spawn_source.map_or(false, |source| {
            matches!(
                source.save_settings_ref(),
                SaveSettings::PreserveDiskTransform
            )
        });

        let entry = HierarchyEntry {
            entity,
            name: name.to_string(),
            entity_type: identity
                .map(|id| id.class.type_abv())
                .unwrap_or_else(|| "Unknown".to_string()),
            parent: relation.map(|r| r.parent()),
            is_expanded: existing_expanded.get(&entity).copied().unwrap_or(false),
            is_dummy_parent: false,
            is_preserve_disk,
            is_preserve_disk_transform,
        };

        if let Some(spawn_source) = spawn_source {
            let file_path = spawn_source.str_ref().to_string();
            file_groups.entry(file_path).or_default().push(entity);
        }

        real_entities.push(entry);
    }

    // Create dummy parents and update hierarchy
    let hierarchy_entries =
        create_file_grouped_hierarchy(real_entities, file_groups, existing_expanded);
    data.hierarchy = hierarchy_entries;
}

/// Creates a hierarchy with file-based dummy parents
fn create_file_grouped_hierarchy(
    real_entities: Vec<HierarchyEntry>,
    file_groups: HashMap<String, Vec<Entity>>,
    existing_expanded: HashMap<Entity, bool>,
) -> Vec<HierarchyEntry> {
    let mut hierarchy_entries: Vec<HierarchyEntry> = Vec::new();
    let mut dummy_parent_entities: HashMap<String, Entity> = HashMap::new();

    for (file_path, entities) in &file_groups {
        if !entities.is_empty() {
            let dummy_entity = create_stable_dummy_entity(file_path);
            dummy_parent_entities.insert(file_path.clone(), dummy_entity);

            let dummy_entry = HierarchyEntry {
                entity: dummy_entity,
                name: file_path.clone(),
                entity_type: "Scene".to_string(),
                parent: None,
                is_expanded: existing_expanded
                    .get(&dummy_entity)
                    .copied()
                    .unwrap_or(true), // Default expanded
                is_dummy_parent: true,
                is_preserve_disk: false,
                is_preserve_disk_transform: false,
            };

            hierarchy_entries.push(dummy_entry);
        }
    }

    for mut entry in real_entities {
        if let Some(spawn_source_path) = file_groups
            .iter()
            .find(|(_, entities)| entities.contains(&entry.entity))
            .map(|(path, _)| path)
        {
            if entry.parent.is_none() {
                if let Some(&dummy_parent) = dummy_parent_entities.get(spawn_source_path) {
                    entry.parent = Some(dummy_parent);
                }
            }
        }

        hierarchy_entries.push(entry);
    }

    sort_hierarchy(&mut hierarchy_entries);
    hierarchy_entries
}

/// Creates a stable dummy entity ID from a file path
fn create_stable_dummy_entity(file_path: &str) -> Entity {
    let mut hash: u32 = 5381;
    for byte in file_path.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash %= 1000000;
    hash += 1; // Ensure non-zero to avoid Entity::from_raw_u32(u32::MAX)
    Entity::from_raw_u32(u32::MAX - hash).expect("u32::Max - anything is valid entity")
}

/// Sorts the hierarchy for consistent display order
fn sort_hierarchy(hierarchy_entries: &mut Vec<HierarchyEntry>) {
    hierarchy_entries.sort_by(|a, b| match (a.is_dummy_parent, b.is_dummy_parent) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (true, true) => a.name.cmp(&b.name),
        (false, false) => a.entity.index().cmp(&b.entity.index()),
    });
}

/// Builds the visual order of entities for tree rendering
pub fn build_visual_order(hierarchy: &[HierarchyEntry]) -> Vec<Entity> {
    let mut children_map: HashMap<Option<Entity>, Vec<Entity>> = HashMap::new();
    for entry in hierarchy {
        children_map
            .entry(entry.parent)
            .or_default()
            .push(entry.entity);
    }

    // Sort children by entity index to maintain consistent order
    for children in children_map.values_mut() {
        children.sort_by_key(|entity| entity.index());
    }

    let expanded_map: HashMap<Entity, bool> = hierarchy
        .iter()
        .map(|entry| (entry.entity, entry.is_expanded))
        .collect();

    let mut visual_order = Vec::new();
    add_visible_children(None, &children_map, &expanded_map, &mut visual_order);
    visual_order
}

/// Recursively adds visible children to the visual order
fn add_visible_children(
    parent: Option<Entity>,
    children_map: &HashMap<Option<Entity>, Vec<Entity>>,
    expanded_map: &HashMap<Entity, bool>,
    visual_order: &mut Vec<Entity>,
) {
    if let Some(children) = children_map.get(&parent) {
        for &child in children {
            visual_order.push(child);

            if expanded_map.get(&child).copied().unwrap_or(false) {
                add_visible_children(Some(child), children_map, expanded_map, visual_order);
            }
        }
    }
}
