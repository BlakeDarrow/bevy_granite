use bevy::prelude::*;

use crate::reflect_serializer::{GarnetSerialize, ReflectGarnetSerialize};

impl GarnetSerialize for Name {
    fn serialize_fmt(&self, stream: &mut dyn std::fmt::Write) -> crate::Result<()> {
        write!(stream, "{:?}", self.as_str())?;
        Ok(())
    }
}

pub fn register_garnet_serialize_types(app: &mut App) {
    app.register_type_data::<Name, ReflectGarnetSerialize>();
}
