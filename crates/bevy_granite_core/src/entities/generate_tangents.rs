use bevy::{
    mesh::{Mesh, Mesh3d},
    prelude::{AssetServer, Assets, Commands, Component, Entity, Query, Res, ResMut, With},
};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

#[derive(Component)]
pub struct NeedsTangents;

/// BevyOBJ doesn't give us tangents, so we need to generate them
pub fn generate_tangents_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Mesh3d), With<NeedsTangents>>,
    asset_server: Res<AssetServer>,
) {
    let mut generated_count = 0;

    for (entity, mesh_handle) in query.iter() {
        if let Some(bevy::asset::LoadState::Loaded) =
            asset_server.get_load_state(mesh_handle.0.id())
        {
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                if mesh.attribute(Mesh::ATTRIBUTE_TANGENT).is_none() {
                    if let Err(e) = mesh.generate_tangents() {
                        log!(
                            LogType::Game,
                            LogLevel::Error,
                            LogCategory::Entity,
                            "Failed to generate tangents: {:?}",
                            e
                        );
                    } else {
                        generated_count += 1;
                    }
                }
                commands.entity(entity).remove::<NeedsTangents>();
            }
        }
    }

    if generated_count > 0 {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::Entity,
            "Total tangents generated: {}",
            generated_count
        );
    }
}
