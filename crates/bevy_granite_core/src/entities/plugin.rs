use std::ops::DerefMut;

use super::{
    despawn_entities_by_source_system, despawn_entities_system, generate_tangents_system,
    BridgeTag, ComponentEditor, HasRuntimeData, IdentityData, InternalNote, MainCamera,
    SpawnSource, UICamera,
};
use crate::entities::{editable::ClassTypePlugin, PromptImportSettings};
use bevy::app::{App, Plugin, Update};

pub struct EntityPlugin;
impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Plugins
            //
            .add_plugins(ClassTypePlugin)
            //
            // Register Types
            // Only register types we want exposed up the plugin chain. These are also duplicate-able
            //
            .register_type::<MainCamera>()
            .register_type::<UICamera>()
            .register_type::<SpawnSource>()
            .register_type_data::<MainCamera, BridgeTag>()
            .register_type::<InternalNote>()
            .register_type_data::<InternalNote, BridgeTag>()
            .register_type::<IdentityData>()
            .register_type::<HasRuntimeData>()
            //
            // Resources
            //
            .insert_resource(ComponentEditor::default())
            .insert_resource(PromptImportSettings::default())
            //
            // Schedule system
            //
            .add_systems(
                Update,
                (
                    despawn_entities_system,
                    despawn_entities_by_source_system,
                    generate_tangents_system,
                ),
            );

        // Add EditorIgnore to bevy types
        super::add_ignore_serialize_to_bevy_types(app);
    }
}
