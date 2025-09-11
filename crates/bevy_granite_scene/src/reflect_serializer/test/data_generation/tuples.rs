use bevy::prelude::*;
use rand::Rng;

use crate::reflect_serializer::test::data_generation::TestEntity;

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
struct TupleComponent((String, f32));

impl TupleComponent {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        match value {
            TestEntity::One => TupleComponent(("Tuple One".to_string(), rng.random())),
            TestEntity::Two => TupleComponent(("Tuple Two".to_string(), rng.random())),
            TestEntity::Three => TupleComponent(("Tuple Three".to_string(), rng.random())),
        }
    }
}

pub fn register_types(registry: &mut App) {
    registry.register_type::<TupleComponent>();
}

pub fn spawn_tuple_entity(
    commands: &mut Commands,
    value: &TestEntity,
    rng: &mut impl Rng,
) -> Entity {
    let component = TupleComponent::get(value, rng);
    commands.spawn((component,)).id()
}
