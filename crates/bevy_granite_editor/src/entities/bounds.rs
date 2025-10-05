use bevy::mesh::{Mesh3d, VertexAttributeValues};
use bevy::prelude::{Assets, Entity, Mesh, Query, Vec3};
use bevy::transform::components::GlobalTransform;
use bevy_granite_core::{ClassCategory, GraniteType, IdentityData};

pub fn get_entity_bounds(
    entity: Entity,
    meshes: &Assets<Mesh>,
    mesh_query: &Query<&Mesh3d>,
) -> Option<(Vec3, Vec3)> {
    let mesh_handle = mesh_query.get(entity).ok()?;
    let mesh = meshes.get(mesh_handle)?;

    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;

    if let VertexAttributeValues::Float32x3(positions) = positions {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        for position in positions.iter() {
            let pos = Vec3::from_array(*position);
            min = min.min(pos);
            max = max.max(pos);
        }

        Some((min, max))
    } else {
        None
    }
}

pub fn get_entity_bounds_world(
    entity: Entity,
    meshes: &Assets<Mesh>,
    mesh_query: &Query<&Mesh3d>,
    transform: &GlobalTransform,
) -> Option<(Vec3, Vec3)> {
    let mesh_handle = mesh_query.get(entity).ok()?;
    let mesh = meshes.get(mesh_handle)?;

    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;

    if let VertexAttributeValues::Float32x3(positions) = positions {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        for position in positions.iter() {
            let pos = transform.transform_point(Vec3::from_array(*position));
            min = min.min(pos);
            max = max.max(pos);
        }

        Some((min, max))
    } else {
        None
    }
}

/// Helper function to get entity bounds, either from mesh data or using a fallback size
pub fn get_entity_bounds_or_fallback(
    entity: Entity,
    identity_data: &IdentityData,
    meshes: &Assets<Mesh>,
    mesh_query: &Query<&Mesh3d>,
) -> Option<(Vec3, Vec3)> {
    match identity_data.class.category() {
        ClassCategory::Mesh => get_entity_bounds(entity, meshes, mesh_query),
        _ => {
            let fake_size = 0.1;
            let half = fake_size / 2.0;
            Some((Vec3::splat(-half), Vec3::splat(half)))
        }
    }
}
