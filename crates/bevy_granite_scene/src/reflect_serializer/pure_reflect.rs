use bevy::{
    prelude::*,
    reflect::{Enum, EnumInfo, TypeInfo, TypeRegistration},
};

pub fn serialize_with_reflect<W: core::fmt::Write>(
    component_data: &dyn Reflect,
    registration: &TypeRegistration,
    info: &TypeInfo,
    stream: &mut W,
    indent: &mut String,
) {
    match component_data.reflect_ref() {
        bevy::reflect::ReflectRef::Enum(e) => {
            enums::serialize_reflect_enum(e, info.as_enum().expect("ref is Enum"), stream, indent);
        }
        bevy::reflect::ReflectRef::Struct(s) => {
            structs::serialize_reflect_struct(
                s,
                info.as_struct().expect("ref is Struct"),
                stream,
                indent,
            );
        }
        _ => {
            _ = writeln!(
                stream,
                "{}: does not ReflectSerialize so it can not be saved yet\n{:?}",
                component_data.reflect_type_path(),
                component_data.reflect_ref().kind()
            );
        }
    }
}

mod enums;
mod structs;
