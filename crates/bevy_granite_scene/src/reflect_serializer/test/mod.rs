use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    ecs::world,
    prelude::*,
    reflect::TypeRegistry,
    render::{primitives::Frustum, view::VisibleEntities},
};
use bevy_granite_core::{EditorIgnore, entities};
use rand::SeedableRng;

use crate::{MetaData, reflect_serializer::test::data_generation::EntitiesForTest};

mod data_generation;

#[test]
fn enums() {
    let mut app = App::new();
    app.insert_resource(Seed(42));
    app.add_systems(Startup, data_generation::spawn_entitys);
    app.init_resource::<EntitiesForTest>();
    init_type_registry(&mut app);
    app.update();
    let world = app.world();
    let entities = world.resource::<data_generation::EntitiesForTest>();
    let mut out = String::new();
    let components = world.components();
    let reg = world.resource::<AppTypeRegistry>().clone();
    let reg = reg.read();
    let meta = MetaData::default();
    let mut entity_serde = super::EntitySerializer::new(&reg, components, &mut out, 0, &meta);
    for (entity, _) in entities.entitys.iter() {
        entity_serde.serialize_entity(*entity, &world);
    }
    println!("Serialized:\n\n\n{}", out);
}

#[test]
fn world_saver() {
    let mut app = App::new();
    app.insert_resource(Seed(42));
    app.init_resource::<EntitiesForTest>();
    app.add_systems(Startup, data_generation::spawn_entitys);
    init_type_registry(&mut app);
    crate::reflect_serializer::register_garnet_serialize_types(&mut app);

    app.update();
    let world = app.world();
    let serialiser = crate::scene::SceneSaver::new(
        world,
        format!(
            "{}/../../assets/scenes/test_scene.garnet",
            std::env::current_dir().unwrap().display()
        ),
    )
    .expect("can create scene saver");
    serialiser.serialize_world().expect("can serialize world");
}

fn init_type_registry(app: &mut App) {
    app.register_type::<Tonemapping>();
    app.register_type::<Name>();
    app.register_type::<Transform>();
    app.register_type::<GlobalTransform>();
    app.register_type::<TransformTreeChanged>();
    app.register_type::<Camera>();
    app.register_type::<Camera3d>();
    app.register_type::<Visibility>();
    app.register_type::<Frustum>();
    app.register_type::<VisibleEntities>();
    app.register_type::<ViewVisibility>();
    app.register_type::<InheritedVisibility>();
    app.register_type::<Msaa>();
    app.register_type::<bevy::render::sync_world::SyncToRenderWorld>();
    bevy_granite_core::entities::add_ignore_serialize_to_bevy_types(app);
    data_generation::register_types(app);
}

#[derive(Resource)]
struct Seed(u64);
