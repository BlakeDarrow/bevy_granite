mod pure_reflect;

// fn serialize_entity<W: Write>(
//     type_registry: &TypeRegistry,
//     entity: EntityRef,
//     components: &Components,
//     stream: &mut W,
//     indent: &mut String,
// ) -> bevy::prelude::Result<()> {
//     // ignore the whole entity if it has the EditorIgnore::SERIALIZE
//     if let Some(ignore) = entity.get::<EditorIgnore>()
//         && ignore.contains(EditorIgnore::SERIALIZE)
//     {
//         return Ok(());
//     }
//     for component_id in entity.archetype().components() {
//         let Some(name) = components.get_name(component_id) else {
//             println!("Failed to get type name for id({component_id:?})");
//             continue;
//         };
//         let Some(registration) = type_registry.get_with_type_path(name.as_ref()) else {
//             println!("Failed to get registration for type name({name})");
//             continue;
//         };
//         // skip components that have the EditorIgnore::SERIALIZE
//         if let Some(ignore) = registration.data::<EditorIgnore>()
//             && ignore.contains(EditorIgnore::SERIALIZE)
//         {
//             continue;
//         }

//         let Some(reflect_componet) = registration.data::<ReflectComponent>() else {
//             println!("Failed to get ReflectComponent for type name({name})");
//             continue;
//         };
//         let Some(component_data) = reflect_componet.reflect(entity) else {
//             println!("Failed to get component data for type name({name})");
//             _ = writeln!(
//                 stream,
//                 "{indent}{name}: does not reflect component so it can not be extracted from the world",
//             );
//             continue;
//         };
//         let Some(serializable) = registration.data::<ReflectSerialize>() else {
//             let info = type_registry
//                 .get_type_info(registration.type_id())
//                 .unwrap_or(component_data.reflect_type_info());
//             serialize_with_reflect(component_data, registration, info, stream, indent);
//             continue;
//         };
//         let serde = serializable.get_serializable(component_data);
//         _ = write!(stream, "{indent}{}: ", component_data.reflect_type_path());
//         let mut serializer = match ron::ser::Serializer::new(
//             &mut *stream,
//             Some(ron::ser::PrettyConfig::default()),
//         ) {
//             Ok(s) => s,
//             Err(e) => {
//                 _ = writeln!(
//                     stream,
//                     "Failed to create ron serializer for type name({name}): {e}"
//                 );
//                 continue;
//             }
//         };
//         if let Err(e) = match serde {
//             bevy::reflect::serde::Serializable::Owned(serialize) => {
//                 Serialize::serialize(&serialize, &mut serializer)
//             }
//             bevy::reflect::serde::Serializable::Borrowed(serialize) => {
//                 Serialize::serialize(serialize, &mut serializer)
//             }
//         } {
//             _ = write!(
//                 stream,
//                 "Failed to serialize component for type name({name}): {e}"
//             );
//             continue;
//         };
//     }
//     Ok(())
// }

// fn deserialize_entity(
//     type_registry: &TypeRegistry,
//     stream: impl Read,
//     world: &mut World,
// ) -> Entity {
//     Entity::PLACEHOLDER
// }

#[cfg(test)]
mod test;

mod human_readable;

use bevy::reflect::reflect_trait;
pub use human_readable::EntitySerializer;

#[derive(thiserror::Error, Debug)]
pub enum ComponentSerializeError {
    #[error("No type registration found for component")]
    NoRegistration,
    #[error("Component is marked with EditorIgnore::SERIALIZE so it will not be serialized")]
    IgnoreSerialize,
    #[error(
        "Component does not implement ReflectComponent so it can not be extracted from the entity"
    )]
    NoReflectComponent,
    #[error("Failed to get component data from entity")]
    NoComponentData,
    #[error("Failed to downcast component to concrete type")]
    DowncastFailed(&'static str),
    #[error("Component does not have type info so it can not be serialized")]
    NoTypeInfo(String),
    #[error("Expected {0} to be of ReflectKind {1}")]
    TypeMissMatch(String, &'static str),
}

#[reflect_trait]
trait GarnetSerialize {
    fn serialize_fmt(&self, stream: &mut dyn std::fmt::Write) -> crate::Result<()>;
}

mod bevy_impls;

pub use bevy_impls::register_garnet_serialize_types;
