use bevy::reflect::StructInfo;

use super::*;

pub fn serialize_reflect_struct<W: core::fmt::Write>(
    component_data: &dyn Struct,
    data: &StructInfo,
    stream: &mut W,
    indent: &mut String,
) {
    _ = writeln!(stream, "{indent}{}: {{", data.ty().type_path_table().path());
    indent.push('\t');
    for field in component_data.iter_fields() {
        // let field_name = field.name().expect("Struct fields must have names");
        // let field_value = field.value();
        // let field_type = data
        //     .field(field_name)
        //     .expect("Field must exist in StructInfo")
        //     .type_info();
        // if let Some(registration) = data
        //     .type_registry()
        //     .get_with_type_path(field_type.type_path_table().path())
        // {
        //     if let Some(_) = registration.data::<ReflectSerialize>() {
        //         serialize_with_reflect(field_value, registration, field_type, stream, indent);
        //         continue;
        //     }
        // }
        // _ = writeln!(
        //     stream,
        //     "{indent}{}: {:?}, // Could not serialize this field",
        //     field_name, field_value
        // );
    }

    indent.pop();
}
