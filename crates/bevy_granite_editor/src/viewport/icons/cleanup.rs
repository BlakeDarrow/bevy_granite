use bevy::{
    ecs::entity::Entity,
    prelude::{Commands, Query, Without},
};
use bevy_granite_core::IconEntity;

pub fn cleanup_icon_entities_system(
    mut commands: Commands,
    icon_query: Query<(Entity, &IconEntity)>,
    target_query: Query<Entity, Without<IconEntity>>,
) {
    for (icon_entity, icon_comp) in icon_query.iter() {
        // If the target entity no longer exists, despawn the icon
        if target_query.get(icon_comp.target_entity).is_err() {
            commands.entity(icon_entity).try_despawn();
        }
    }
}
