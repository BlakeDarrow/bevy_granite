use bevy::{ecs::entity::EntityHashMap, prelude::*, reflect::TypeRegistry};
use bevy_granite_core::entities::ExposedToEditor;

mod reflect_serializer;
mod scene;

type Result<T> = std::result::Result<T, scene::SceneFormatError>;
type MetaData = EntityHashMap<scene::EntityMetaData>;

#[macro_export]
macro_rules! pwrite {
    ($stream:expr, $str:literal:$pad:expr, $($arg:tt)+) => {
        write!($stream, "{}{}", "\t".repeat($pad), format!($str, $($arg)+))
    };
    ($stream:expr, $str:literal:$pad:expr) => {
        write!($stream, "{}{}", "\t".repeat($pad), $str)
    };
    ($stream:expr, $pad:expr) => {
        write!($stream, "{}", "\t".repeat($pad))
    };
}

#[macro_export]
macro_rules! pwriteln {
    ($stream:expr, $str:literal:$pad:expr, $($arg:tt)+) => {
        writeln!($stream, "{}{}", "\t".repeat($pad), format!($str, $($arg)+))
    };
    ($stream:expr, $str:literal:$pad:expr) => {
        writeln!($stream, "{}{}", "\t".repeat($pad), $str)
    };
}
