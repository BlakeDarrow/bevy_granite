use super::*;

pub fn register_types(registry: &mut App) {
    registry.register_type::<UnitEnum>();
    registry.register_type::<TupleEnum>();
    registry.register_type::<StructEnum>();
    registry.register_type::<MixedEnum>();
    registry.register_type::<NonTrivialEnum>();
}

#[derive(Reflect, Component, Clone)]
#[reflect(Component)]
enum UnitEnum {
    A,
    B,
    C,
}

impl UnitEnum {
    fn get(value: &TestEntity) -> Self {
        match value {
            TestEntity::One => UnitEnum::A,
            TestEntity::Two => UnitEnum::B,
            TestEntity::Three => UnitEnum::C,
        }
    }
}

#[derive(Reflect, Component, Clone)]
#[reflect(Component)]
enum TupleEnum {
    Empty(),
    D(u32),
    E(f32, f32),
    F(String, String, String),
}

impl TupleEnum {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        match value {
            TestEntity::One => TupleEnum::D(rng.next_u32()),
            TestEntity::Two => TupleEnum::E(rng.random(), rng.random()),
            TestEntity::Three => TupleEnum::F(
                TEST_WORDS.choose(rng).unwrap().to_string(),
                TEST_WORDS.choose(rng).unwrap().to_string(),
                TEST_WORDS.choose(rng).unwrap().to_string(),
            ),
        }
    }
}

#[derive(Reflect, Component, Clone)]
#[reflect(Component)]
enum StructEnum {
    Empty {},
    G { q: u32 },
    H { a: f32, b: f32 },
    I { x: String, y: String, z: String },
}

impl StructEnum {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        match value {
            TestEntity::One => StructEnum::G { q: rng.next_u32() },
            TestEntity::Two => StructEnum::H {
                a: rng.random(),
                b: rng.random(),
            },
            TestEntity::Three => StructEnum::I {
                x: TEST_WORDS.choose(rng).unwrap().to_string(),
                y: TEST_WORDS.choose(rng).unwrap().to_string(),
                z: TEST_WORDS.choose(rng).unwrap().to_string(),
            },
        }
    }
}

#[derive(Reflect, Component, Clone)]
#[reflect(Component)]
enum MixedEnum {
    Unit,
    Tuple(u32),
    Struct { value: String },
}

impl MixedEnum {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        match value {
            TestEntity::One => MixedEnum::Unit,
            TestEntity::Two => MixedEnum::Tuple(rng.next_u32()),
            TestEntity::Three => MixedEnum::Struct {
                value: TEST_WORDS.choose(rng).unwrap().to_string(),
            },
        }
    }
}

#[derive(Reflect, Component, Clone)]
#[reflect(Component)]
enum NonTrivialEnum {
    Transform {
        value: Transform,
    },
    Name {
        value: Name,
    },
    Tonemapping {
        value: bevy::core_pipeline::tonemapping::Tonemapping,
    },
}

pub fn add_test_entity(
    world: &mut Commands,
    value: &TestEntity,
    rng: &mut impl Rng,
    ground_truth: &mut Vec<Box<dyn Reflect>>,
) -> Entity {
    let unit = UnitEnum::get(value);
    let tuple = TupleEnum::get(value, rng);
    let struct_ = StructEnum::get(value, rng);
    let mixed = MixedEnum::get(value, rng);
    let non_trivial = match value {
        TestEntity::One => NonTrivialEnum::Transform {
            value: Transform::from_translation(Vec3::new(rng.random(), rng.random(), rng.random())),
        },
        TestEntity::Two => NonTrivialEnum::Name {
            value: Name::new(format!("Entity: {}", rng.random::<u8>())),
        },
        TestEntity::Three => NonTrivialEnum::Tonemapping {
            value: bevy::core_pipeline::tonemapping::Tonemapping::default(),
        },
    };
    let transform =
        Transform::from_translation(Vec3::new(rng.random(), rng.random(), rng.random()));
    let name = Name::new(format!("Entity: {}", rng.random::<u8>()));

    ground_truth.push(unit.reflect_clone().unwrap());
    ground_truth.push(tuple.reflect_clone().unwrap());
    ground_truth.push(struct_.reflect_clone().unwrap());
    ground_truth.push(mixed.reflect_clone().unwrap());
    ground_truth.push(transform.reflect_clone().unwrap());
    ground_truth.push(name.reflect_clone().unwrap());

    world
        .spawn((name, unit, tuple, struct_, mixed, transform, non_trivial))
        .id()
}
