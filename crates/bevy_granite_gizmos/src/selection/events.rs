use bevy::{
    ecs::event::Event,
    prelude::{Entity, Message},
};

#[derive(Event)]
pub enum EntityEvents {
    Select { target: Entity, additive: bool },
    SelectRange { range: Vec<Entity>, additive: bool },
    Deselect { target: Entity },
    DeselectRange { range: Vec<Entity> },
    DeselectAll,
}

#[derive(Message)]
pub struct RequestDuplicateEntityEvent {
    pub entity: Entity,
}

#[derive(Message)]
pub struct RequestDuplicateAllSelectionEvent;
