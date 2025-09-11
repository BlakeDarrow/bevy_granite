use bevy::ecs::reflect::ReflectComponent;
use bevy::ecs::world::EntityRef;
use bevy::reflect::{Reflect, TypeRegistry};
use bevy_granite_core::EditorIgnore;

use crate::reflect_serializer::ComponentSerializeError;
use crate::{MetaData, pwrite};

pub struct ComponentSerializer<'a, W> {
    type_registry: &'a TypeRegistry,
    stream: &'a mut W,
    indent: usize,
    metadata: &'a MetaData,
}

impl<'a, W> ComponentSerializer<'a, W> {
    pub fn new(
        type_registry: &'a TypeRegistry,
        stream: &'a mut W,
        indent: usize,
        metadata: &'a MetaData,
    ) -> Self {
        Self {
            type_registry,
            stream,
            indent,
            metadata,
        }
    }
}

impl<'a, W: std::fmt::Write> ComponentSerializer<'a, W> {
    pub fn serialize_component(&mut self, name: &str, entity: EntityRef) -> crate::Result<()> {
        // get the type registration for the component
        let Some(registration) = self.type_registry.get_with_type_path(name) else {
            // skip a component if they have no registered it
            return Err(ComponentSerializeError::NoRegistration.into());
        };
        // check if the component has EditorIgnore::SERIALIZE
        // if so this is considered a runtime only component and that parsisting its data is unwanted
        if let Some(ignore) = registration.data::<EditorIgnore>()
            && ignore.contains(EditorIgnore::SERIALIZE)
        {
            return Ok(());
        }

        // get reflect component to extract the component data from the entity
        let Some(reflect_component) = registration.data::<ReflectComponent>() else {
            return Err(ComponentSerializeError::NoReflectComponent.into());
        };

        let Some(component_data) = reflect_component.reflect(entity) else {
            return Err(ComponentSerializeError::NoComponentData.into());
        };
        // print the component type name
        pwrite!(self.stream, "{}: ":self.indent, component_data.reflect_type_path())?;

        let mut serializer = super::ReflectSerializer::new(
            self.type_registry,
            self.stream,
            self.indent,
            self.metadata,
        );

        serializer.serialize_inline(component_data)?;
        writeln!(self.stream, ";")?;
        Ok(())
    }
}
