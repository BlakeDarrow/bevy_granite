use bevy::prelude::*;
use rand::{Rng, seq::IndexedRandom};

use crate::reflect_serializer::test::data_generation::{TEST_WORDS, TestEntity};

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
struct MapComponet(std::collections::HashMap<String, i32>);

impl MapComponet {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        let mut map = std::collections::HashMap::new();
        match value {
            TestEntity::One => {
                map.insert("One".to_string(), rng.random_range(0..10));
                map.insert("Two".to_string(), rng.random_range(0..10));
            }
            TestEntity::Two => {
                map.insert("Three".to_string(), rng.random_range(10..20));
                map.insert("Four".to_string(), rng.random_range(10..20));
            }
            TestEntity::Three => {
                map.insert("Five".to_string(), rng.random_range(20..30));
                map.insert("Six".to_string(), rng.random_range(20..30));
            }
        }
        MapComponet(map)
    }
}

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
struct ArrayComponet([i32; 5]);

impl ArrayComponet {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        match value {
            TestEntity::One => ArrayComponet([
                rng.random_range(0..10),
                rng.random_range(0..10),
                rng.random_range(0..10),
                rng.random_range(0..10),
                rng.random_range(0..10),
            ]),
            TestEntity::Two => ArrayComponet([
                rng.random_range(10..20),
                rng.random_range(10..20),
                rng.random_range(10..20),
                rng.random_range(10..20),
                rng.random_range(10..20),
            ]),
            TestEntity::Three => ArrayComponet([
                rng.random_range(20..30),
                rng.random_range(20..30),
                rng.random_range(20..30),
                rng.random_range(20..30),
                rng.random_range(20..30),
            ]),
        }
    }
}

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
struct SetComponet(std::collections::HashSet<String>);

impl SetComponet {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        let mut set = std::collections::HashSet::new();
        match value {
            TestEntity::One => {
                set.insert("One".to_string());
                set.insert("Two".to_string());
            }
            TestEntity::Two => {
                set.insert("Three".to_string());
                set.insert("Four".to_string());
            }
            TestEntity::Three => {
                set.insert("Five".to_string());
                set.insert("Six".to_string());
            }
        }
        SetComponet(set)
    }
}

#[derive(Reflect, Default, Component)]
#[reflect(Component)]
struct ListComponet(Vec<String>);

impl ListComponet {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        match value {
            TestEntity::One => ListComponet(vec![
                TEST_WORDS.choose(rng).unwrap().to_string(),
                TEST_WORDS.choose(rng).unwrap().to_string(),
                TEST_WORDS.choose(rng).unwrap().to_string(),
            ]),
            TestEntity::Two => ListComponet(vec![
                TEST_WORDS.choose(rng).unwrap().to_string(),
                TEST_WORDS.choose(rng).unwrap().to_string(),
                TEST_WORDS.choose(rng).unwrap().to_string(),
            ]),
            TestEntity::Three => ListComponet(vec![
                TEST_WORDS.choose(rng).unwrap().to_string(),
                TEST_WORDS.choose(rng).unwrap().to_string(),
                TEST_WORDS.choose(rng).unwrap().to_string(),
            ]),
        }
    }
}

pub fn register_types(registry: &mut App) {
    registry.register_type::<MapComponet>();
    registry.register_type::<ArrayComponet>();
    registry.register_type::<SetComponet>();
    registry.register_type::<ListComponet>();
}

pub fn add_test_entity(
    world: &mut Commands,
    value: &TestEntity,
    rng: &mut impl Rng,
    ground_truth: &mut Vec<Box<dyn Reflect>>,
) -> Entity {
    let map = MapComponet::get(value, rng);
    let array = ArrayComponet::get(value, rng);
    let set = SetComponet::get(value, rng);
    let list = ListComponet::get(value, rng);

    ground_truth.push(map.reflect_clone().unwrap());
    ground_truth.push(set.reflect_clone().unwrap());
    ground_truth.push(list.reflect_clone().unwrap());

    world.spawn((map, array, set, list)).id()
}
