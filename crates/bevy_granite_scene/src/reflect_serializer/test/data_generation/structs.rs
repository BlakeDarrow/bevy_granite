use super::*;

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct UnitStruct;

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct NewType(pub i32);

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct AnotherType {
    pub a: f32,
    pub b: Transform,
}

pub fn register_types(registry: &mut App) {
    registry.register_type::<UnitStruct>();
    registry.register_type::<NewType>();
    registry.register_type::<AnotherType>();
}

impl NewType {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        NewType(rng.random_range(0..1000)) // Random value between 0 and 999
    }
}

impl AnotherType {
    fn get<T: Rng>(value: &TestEntity, rng: &mut T) -> Self {
        match value {
            TestEntity::One => AnotherType {
                a: rng.random(),
                b: Transform::from_translation(Vec3::splat(rng.random())),
            },
            TestEntity::Two => AnotherType {
                a: rng.random(),
                b: Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, rng.random())),
            },
            TestEntity::Three => AnotherType {
                a: rng.random(),
                b: Transform::from_scale(Vec3::splat(rng.random())),
            },
        }
    }
}

pub fn add_test_entity(
    world: &mut Commands,
    value: &TestEntity,
    rng: &mut impl Rng,
    ground_truth: &mut Vec<Box<dyn Reflect>>,
) -> Entity {
    let unit = UnitStruct;
    let tuple = NewType::get(value, rng);
    let struct_ = AnotherType::get(value, rng);
    let name = Name::new(format!("Entity: {}", rng.random::<u8>()));

    ground_truth.push(unit.reflect_clone().unwrap());
    ground_truth.push(tuple.reflect_clone().unwrap());
    ground_truth.push(struct_.reflect_clone().unwrap());
    ground_truth.push(name.reflect_clone().unwrap());

    world
        .spawn((name, unit, tuple, struct_, Camera3d::default()))
        .id()
}
