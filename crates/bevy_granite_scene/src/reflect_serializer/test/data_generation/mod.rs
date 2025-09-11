use bevy::reflect::TypeRegistry;
use bevy::{ecs::world, prelude::*};
use bevy_granite_core::entities;
use rand::prelude::*;

use crate::reflect_serializer::test::Seed;
const TEST_WORDS: [&str; 130] = [
    "apple",
    "banana",
    "cherry",
    "date",
    "elderberry",
    "fig",
    "grape",
    "honeydew",
    "kiwi",
    "lemon",
    "mango",
    "nectarine",
    "orange",
    "papaya",
    "quince",
    "raspberry",
    "strawberry",
    "tangerine",
    "uglifruit",
    "vanilla",
    "xigua",
    "yuzu",
    "zucchini",
    "apricot",
    "blackberry",
    "cantaloupe",
    "damson",
    "eggplant",
    "feijoa",
    "gooseberry",
    "huckleberry",
    "imbe",
    "jackfruit",
    "kumquat",
    "lime",
    "mulberry",
    "nashi",
    "olive",
    "peach",
    "plum",
    "raisin",
    "satsuma",
    "tomato",
    "uva",
    "watermelon",
    "boysenberry",
    "cloudberry",
    "dragonfruit",
    "elephantapple",
    "falsejujube",
    "genip",
    "hornedmelon",
    "ilama",
    "jambolan",
    "kaffirlime",
    "longan",
    "mamey",
    "nance",
    "orangelo",
    "persimmon",
    "rambutan",
    "sapodilla",
    "tamarind",
    "umbrellafruit",
    "voavanga",
    "wolfberry",
    "ximenia",
    "yumberry",
    "zalacca",
    "amla",
    "bilberry",
    "cranberry",
    "durian",
    "eugenia",
    "figfruit",
    "guava",
    "hawthorn",
    "icaco",
    "jujube",
    "kiwiberry",
    "lychee",
    "mandarine",
    "nutmeg",
    "ovalkumquat",
    "pomegranate",
    "quararibea",
    "roseapple",
    "salal",
    "tangelo",
    "ume",
    "vandachrysanthemum",
    "whitecurrant",
    "yuzu",
    "zapotefruit",
    "ackee",
    "breadfruit",
    "cacao",
    "dovyalis",
    "entawak",
    "fangfruit",
    "goji",
    "hogplum",
    "illyrianplum",
    "juneberry",
    "kaong",
    "loquat",
    "miraclefruit",
    "nutmegmelon",
    "oroblanco",
    "pawpaw",
    "quinceapple",
    "rambutanpear",
    "salak",
    "tamarillo",
    "uluguru",
    "vochysia",
    "waxapple",
    "yewplum",
    "zambalesmango",
    "alkanet",
    "bayberry",
    "currant",
    "dateplum",
    "eggfruit",
    "flacourtia",
    "guineapeach",
    "huckle",
    "imbu",
    "jaboticaba",
    "knobthorn",
];

pub enum TestEntity {
    One,
    Two,
    Three,
}

impl TestEntity {
    fn iter() -> impl Iterator<Item = TestEntity> {
        [TestEntity::One, TestEntity::Two, TestEntity::Three].into_iter()
    }
}

mod collections;
mod enums;
mod structs;
mod tuples;
pub fn register_types(registry: &mut App) {
    enums::register_types(registry);
    structs::register_types(registry);
    tuples::register_types(registry);
    collections::register_types(registry);
}

pub fn spawn_entitys(
    mut commands: Commands,
    rng: Res<Seed>,
    mut entities: ResMut<EntitiesForTest>,
) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(rng.0);
    for test in TestEntity::iter() {
        let tuple = tuples::spawn_tuple_entity(&mut commands, &test, &mut rng);
        entities.entitys.push((tuple, vec![]));
        let mut ground_truth = Vec::new();
        let entity = enums::add_test_entity(&mut commands, &test, &mut rng, &mut ground_truth);
        entities.entitys.push((entity, ground_truth));
        let mut ground_truth = Vec::new();
        let entity2 = structs::add_test_entity(&mut commands, &test, &mut rng, &mut ground_truth);
        entities.entitys.push((entity2, ground_truth));
        let mut ground_truth = Vec::new();
        let entity3 =
            collections::add_test_entity(&mut commands, &test, &mut rng, &mut ground_truth);
        entities.entitys.push((entity3, ground_truth));
        commands.entity(entity).add_child(entity2);
        commands.entity(entity).add_child(entity3);
        commands.entity(entity).add_child(tuple);
    }
}

#[derive(Resource, Default)]
pub struct EntitiesForTest {
    pub entitys: Vec<(Entity, Vec<Box<dyn Reflect>>)>,
}
