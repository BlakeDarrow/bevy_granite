use super::*;

pub fn serialize_reflect_enum<W: core::fmt::Write>(
    component_data: &dyn Enum,
    data: &EnumInfo,
    stream: &mut W,
    indent: &mut String,
) {
    _ = write!(stream, "{indent}{}: ", data.ty().type_path_table().path());
    match component_data.variant_type() {
        bevy::reflect::VariantType::Unit => {
            _ = writeln!(stream, "{},", component_data.variant_name());
        }
        bevy::reflect::VariantType::Tuple => {
            _ = write!(stream, "{}(", component_data.variant_name());
            let mut first = true;
            for field in component_data.iter_fields() {
                if !first {
                    _ = write!(stream, ", ");
                }
                first = false;
                _ = write!(stream, "{:?}", field.value());
            }
            _ = writeln!(stream, "),");
        }
        bevy::reflect::VariantType::Struct => {
            _ = writeln!(stream, "{} {{", component_data.variant_name());
            for field in component_data.iter_fields() {
                _ = writeln!(
                    stream,
                    "{indent}\t{}: {:?},",
                    field.name().expect("Struct fields must have names"),
                    field.value()
                );
            }
            _ = writeln!(stream, "{indent}}},");
        }
    }
}
