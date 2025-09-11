use bevy::{
    asset::uuid,
    ecs::{
        component::{ComponentId, Components},
        entity::{Entity, EntityHashMap, EntityHashSet},
        reflect::AppTypeRegistry,
        world::World,
    },
    reflect::TypeRegistry,
};
use bevy_granite_core::{EditorIgnore, entities::serialize::SceneMetadata};

use crate::{MetaData, Result};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SceneData {
    pub metadata: SceneMetadata,
    pub entities: Vec<SaveReadyEntity>,
    pub resources: Vec<ResourceSaveReadyData>,
}

impl SceneData {
    pub fn new(entity_count: usize) -> Self {
        Self {
            metadata: SceneMetadata {
                format_version: bevy_granite_core::get_beta_scene_version(),
                entity_count,
            },
            entities: Vec::with_capacity(entity_count),
            resources: Vec::new(),
        }
    }
}

pub struct EntityMetaData {
    pub id: uuid::Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SaveReadyEntity {}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ResourceSaveReadyData {}

pub struct SceneSaver<'a, W> {
    entity_map: MetaData,
    register: AppTypeRegistry,
    components: &'a Components,
    world: &'a World,
    indent: usize,
    format: SceneFormat,
    file: W,
}

pub struct IoWapper<W: std::io::Write> {
    inner: W,
}

impl<W: std::io::Write> core::fmt::Write for IoWapper<W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.inner
            .write_all(s.as_bytes())
            .map_err(|_| std::fmt::Error)
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.inner
            .write_all(c.encode_utf8(&mut [0; 4]).as_bytes())
            .map_err(|_| std::fmt::Error)
    }
    fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> std::fmt::Result {
        self.inner.write_fmt(args).map_err(|_| std::fmt::Error)
    }
}

impl<'a> SceneSaver<'a, IoWapper<std::io::BufWriter<std::fs::File>>> {
    pub fn new(world: &'a World, file: impl AsRef<str>) -> std::io::Result<Self> {
        let abs = bevy_granite_core::shared::rel_asset_to_absolute(file.as_ref());
        println!("Saving scene to absolute path: {abs}");
        let file = std::fs::File::create(abs.as_ref())?;
        let writer = std::io::BufWriter::new(file);
        let writer = IoWapper { inner: writer };
        Ok(Self {
            entity_map: EntityHashMap::default(),
            file: writer,
            register: world.resource::<AppTypeRegistry>().clone(),
            components: world.components(),
            world,
            format: SceneFormat::HumanVerbose,
            indent: 0,
        })
    }
}

impl<'a, W> SceneSaver<'a, W> {
    #[inline]
    pub fn reserve_entity(&mut self, entity: Entity) {
        let id = uuid::Uuid::new_v4();
        self.entity_map.insert(entity, EntityMetaData { id });
    }

    #[inline]
    pub fn reserve_entities<I: IntoIterator<Item = Entity>>(&mut self, entities: I) {
        for entity in entities {
            self.reserve_entity(entity);
        }
    }
    #[inline]
    pub fn ron_config() -> ron::ser::PrettyConfig {
        ron::ser::PrettyConfig::new()
            .depth_limit(15)
            .separate_tuple_members(false)
            .enumerate_arrays(false)
            .compact_arrays(true)
            .indentor("\t".to_string())
    }

    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entity_map.len()
    }

    #[inline]
    pub fn resource_count(&self) -> usize {
        0 //TODO
    }
}
impl<'a, W: std::fmt::Write> SceneSaver<'a, W> {
    pub fn serialize_world(mut self) -> bevy::prelude::Result<()> {
        // build entity -> uuid map
        let maybe_skip = self
            .world
            .component_id::<EditorIgnore>()
            .unwrap_or(ComponentId::new(usize::MAX));
        for archetype in self.world.archetypes().iter() {
            if !archetype.contains(maybe_skip) {
                self.reserve_entities(archetype.entities().iter().map(|e| e.id()));
                continue;
            }
            for entity in archetype.entities() {
                if let Some(ignore_flags) = self.world.entity(entity.id()).get::<EditorIgnore>()
                    && !ignore_flags.contains(EditorIgnore::SERIALIZE)
                {
                    self.reserve_entity(entity.id());
                }
            }
        }
        //add metadata to .garnet
        self.add_metadata()?;

        if self.entity_count() > 0 {
            self.serialize_entities()?;
        }
        Ok(())
    }

    fn add_metadata(&mut self) -> Result<()> {
        write!(
            &mut self.file,
            r#"{}
[metadata]
format_version: {};
"#,
            self.format.magic(),
            bevy_granite_core::get_beta_scene_version()
        )?;

        let count = self.entity_count();
        if count > 0 {
            writeln!(&mut self.file, "entity_count: {count}")?;
        }

        let count = self.resource_count();
        if count > 0 {
            writeln!(&mut self.file, "resource_count: {count}")?;
        }

        Ok(())
    }

    fn serialize_entities(&mut self) -> Result<()> {
        writeln!(&mut self.file, "[entities]")?;
        let register = self.register.read();
        let mut entitiy_serializer = crate::reflect_serializer::EntitySerializer::new(
            &register,
            self.components,
            &mut self.file,
            self.indent + 1,
            &self.entity_map,
        );
        for entity in self.entity_map.keys() {
            entitiy_serializer.serialize_entity(*entity, self.world)?;
        }
        Ok(())
    }
}

enum SceneFormat {
    HumanVerbose,
    HumanCompact,
    Binary,
}

impl SceneFormat {
    fn magic(&self) -> &str {
        match self {
            SceneFormat::HumanVerbose => "GHS",
            SceneFormat::HumanCompact => "GHS",
            SceneFormat::Binary => "GBS",
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SceneFormatError {
    #[error("Unknown format: {0}")]
    UnknownFormat(String),
    #[error("Format write error: {0}")]
    FmtWriteError(#[from] std::fmt::Error),
    #[error(
        "Entity {0:?} has no metadata, this means it doesn't have a associated UUID, this might be automatic in future but for now all entities must be reserved before serializing"
    )]
    EntityNotReserved(Entity),
    #[error("component serialization error: {0}")]
    ComponentSerializeError(#[from] crate::reflect_serializer::ComponentSerializeError),
}
