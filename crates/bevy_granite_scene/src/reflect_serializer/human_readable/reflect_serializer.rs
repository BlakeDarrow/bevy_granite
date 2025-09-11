use bevy::{
    ecs::entity::Entity,
    reflect::{
        Array, Enum, EnumInfo, List, Map, PartialReflect, Reflect, ReflectRef, Set, Struct,
        TupleStruct, TypePath, TypeRegistry,
    },
};

use crate::{
    MetaData, pwrite, pwriteln,
    reflect_serializer::{ComponentSerializeError, ReflectGarnetSerialize},
};

pub struct ReflectSerializer<'a, W> {
    type_registry: &'a TypeRegistry,
    stream: &'a mut W,
    indent: usize,
    metadata: &'a MetaData,
}

impl<'a, W> ReflectSerializer<'a, W> {
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

impl<'a, W: std::fmt::Write> ReflectSerializer<'a, W> {
    /// adds the appropriate amount of padding before serializing the object
    #[inline(always)]
    pub fn serialize(&mut self, object: &dyn PartialReflect) -> crate::Result<()> {
        pwrite!(self.stream, self.indent)?;
        self.serialize_inline(object)
    }

    /// skips adding padding on the first line of serialization
    /// this is useful for keeping single line objects on a single line
    /// and only adding padding to subsequent lines
    pub fn serialize_inline(&mut self, object: &dyn PartialReflect) -> crate::Result<()> {
        if let Some(custom) = self
            .type_registry
            .get_with_type_path(object.reflect_type_path())
            .and_then(|r| r.data::<ReflectGarnetSerialize>())
            && let Some(garnet) = object.try_as_reflect().and_then(|full| custom.get(full))
        {
            return garnet.serialize_fmt(self.stream);
        }
        match object.reflect_ref() {
            ReflectRef::Enum(e) => {
                self.serialize_enum(e)?;
            }
            ReflectRef::Opaque(e) => {
                self.serialize_opaque(e)?;
            }
            ReflectRef::TupleStruct(s) => {
                self.serialize_tuple_struct(s)?;
            }
            ReflectRef::Struct(s) => {
                self.serialize_struct(s)?;
            }
            ReflectRef::List(l) => {
                self.serialize_list(l)?;
            }
            ReflectRef::Tuple(t) => {
                self.serialize_tuple(t)?;
            }
            ReflectRef::Array(a) => {
                self.serialize_array(a)?;
            }
            ReflectRef::Map(m) => {
                self.serialize_map(m)?;
            }
            ReflectRef::Set(set) => {
                self.serialize_set(set)?;
            }
        }
        Ok(())
    }

    fn serialize_enum(&mut self, enum_ref: &dyn Enum) -> crate::Result<()> {
        write!(self.stream, "{}", enum_ref.variant_name())?;
        match enum_ref.variant_type() {
            bevy::reflect::VariantType::Unit => {}
            bevy::reflect::VariantType::Tuple => {
                write!(self.stream, "(")?;
                let mut first = true;
                let inline = enum_ref.field_len() <= 3;
                for field in enum_ref.iter_fields() {
                    if !first {
                        if inline {
                            write!(self.stream, ", ")?;
                        } else {
                            writeln!(self.stream, ",")?;
                        }
                    }
                    if inline {
                        self.serialize_inline(field.value())?;
                    } else {
                        self.serialize(field.value())?;
                    }
                    first = false;
                }
                if inline {
                    write!(self.stream, ")")?;
                } else {
                    pwrite!(self.stream, ")":self.indent);
                }
            }
            bevy::reflect::VariantType::Struct => {
                writeln!(self.stream, " {{")?;
                for field in enum_ref.iter_fields() {
                    if let Some(name) = field.name() {
                        pwrite!(self.stream, "{}: ":self.indent+1, name);
                    } else {
                        pwrite!(self.stream, "{}: ":self.indent+1, "<unnamed>");
                    };
                    self.serialize_inline(field.value())?;
                    writeln!(self.stream, ",")?;
                }
                pwrite!(self.stream, "}":self.indent)?;
            }
        }

        Ok(())
    }

    fn serialize_opaque(&mut self, opaque_ref: &dyn PartialReflect) -> crate::Result<()> {
        if opaque_ref.represents::<Entity>() {
            let Some(entity) = opaque_ref.try_downcast_ref::<Entity>() else {
                return Err(
                    crate::reflect_serializer::ComponentSerializeError::DowncastFailed(
                        Entity::type_path(),
                    )
                    .into(),
                );
            };
            let Some(entity) = self.metadata.get(entity) else {
                return Err(crate::scene::SceneFormatError::EntityNotReserved(*entity));
            };
            write!(self.stream, "{}", entity.id)?;
            return Ok(());
        }
        write!(self.stream, "{opaque_ref:?}")?;
        Ok(())
    }

    fn serialize_tuple_struct(&mut self, struct_ref: &dyn TupleStruct) -> crate::Result<()> {
        if struct_ref.field_len() == 1 {
            self.serialize_inline(struct_ref.field(0).expect("expect at least one field"))?;
            return Ok(());
        }
        write!(self.stream, "(")?;
        let mut first = true;
        let inline = struct_ref.field_len() <= 3;
        if !inline {
            self.stream.write_char('\n')?;
        }
        for field in struct_ref.iter_fields() {
            if !first {
                if inline {
                    write!(self.stream, ", ")?;
                } else {
                    writeln!(self.stream, ",")?;
                }
            }
            self.serialize_inline(field)?;
            first = false;
        }
        if inline {
            write!(self.stream, ")")?;
        } else {
            pwrite!(self.stream, "\n)":self.indent)?;
        }
        Ok(())
    }

    fn serialize_struct(&mut self, struct_ref: &dyn Struct) -> crate::Result<()> {
        if struct_ref.field_len() == 0 {
            write!(self.stream, "()")?;
            return Ok(());
        }
        writeln!(self.stream, "{{")?;
        let Some(info) = struct_ref.get_represented_type_info() else {
            return Err(crate::scene::SceneFormatError::ComponentSerializeError(
                crate::reflect_serializer::ComponentSerializeError::NoTypeInfo(
                    struct_ref.reflect_type_path().to_string(),
                ),
            ));
        };
        self.indent += 1;
        for (i, field) in info
            .as_struct()
            .map_err(|_| {
                ComponentSerializeError::TypeMissMatch(
                    struct_ref.reflect_type_path().to_string(),
                    "Struct",
                )
            })?
            .iter()
            .enumerate()
        {
            pwrite!(self.stream, "{}: ":self.indent, field.name())?;
            let Some(value) = struct_ref.field_at(i) else {
                panic!("field index out of bounds");
            };
            self.serialize_inline(value)?;
            writeln!(self.stream, ",")?;
        }
        self.indent -= 1;
        pwrite!(self.stream, "}":self.indent)?;
        Ok(())
    }

    fn serialize_list(&mut self, list_ref: &dyn List) -> crate::Result<()> {
        write!(self.stream, "[")?;
        if list_ref.is_empty() {
            write!(self.stream, "]")?;
            return Ok(());
        }
        self.stream.write_char('\n')?;
        let mut first = true;
        self.indent += 1;
        for item in list_ref.iter() {
            if !first {
                writeln!(self.stream, ",")?;
            }
            self.serialize(item)?;
            first = false;
        }
        self.stream.write_char('\n')?;
        self.indent -= 1;
        pwrite!(self.stream, "]":self.indent)?;
        Ok(())
    }

    fn serialize_array(&mut self, array_ref: &dyn Array) -> crate::Result<()> {
        write!(self.stream, "[")?;
        if array_ref.is_empty() {
            write!(self.stream, "]")?;
            return Ok(());
        }
        self.stream.write_char('\n')?;
        let mut first = true;
        self.indent += 1;
        for item in array_ref.iter() {
            if !first {
                writeln!(self.stream, ",")?;
            }
            self.serialize(item)?;
            first = false;
        }
        self.indent -= 1;
        self.stream.write_char('\n')?;
        pwrite!(self.stream, "]":self.indent)?;
        Ok(())
    }

    fn serialize_set(&mut self, set_ref: &dyn Set) -> crate::Result<()> {
        write!(self.stream, "{{")?;
        if set_ref.is_empty() {
            write!(self.stream, "}}")?;
            return Ok(());
        }
        self.stream.write_char('\n')?;
        let mut first = true;
        self.indent += 1;
        for item in set_ref.iter() {
            if !first {
                writeln!(self.stream, ",")?;
            }
            self.serialize(item)?;
            first = false;
        }
        self.indent -= 1;
        self.stream.write_char('\n')?;
        pwrite!(self.stream, "}":self.indent)?;
        Ok(())
    }

    fn serialize_map(&mut self, map_ref: &dyn Map) -> crate::Result<()> {
        write!(self.stream, "{{")?;
        if map_ref.is_empty() {
            write!(self.stream, "}}")?;
            return Ok(());
        }
        self.stream.write_char('\n')?;
        self.indent += 1;
        for (i, (key, value)) in map_ref.iter().enumerate() {
            if i > 0 {
                writeln!(self.stream, ",")?;
            }
            self.serialize(key)?;
            write!(self.stream, ": ")?;
            self.serialize_inline(value)?;
        }
        self.indent -= 1;
        self.stream.write_char('\n')?;
        pwrite!(self.stream, "}":self.indent)?;
        Ok(())
    }

    fn serialize_tuple(&mut self, tuple_ref: &dyn bevy::reflect::Tuple) -> crate::Result<()> {
        if tuple_ref.field_len() == 0 {
            write!(self.stream, "()")?;
            return Ok(());
        }
        if tuple_ref.field_len() == 1 {
            self.serialize_inline(tuple_ref.field(0).expect("expect at least one field"))?;
            return Ok(());
        }
        writeln!(self.stream, "(")?;
        for i in 0..tuple_ref.field_len() {
            if i > 0 {
                writeln!(self.stream, ",")?;
            }
            self.serialize(tuple_ref.field(i).expect("field index out of bounds"))?;
        }
        self.stream.write_char('\n')?;
        pwrite!(self.stream, ")":self.indent)?;
        Ok(())
    }
}
