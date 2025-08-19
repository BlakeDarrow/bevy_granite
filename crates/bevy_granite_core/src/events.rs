use bevy::prelude::Event;


#[derive(Event)]
pub struct RuntimeDataReadyEvent(pub String);

#[derive(Event)]
pub struct CollectRuntimeDataEvent(pub String);

#[derive(Event)]
pub struct WorldLoadSuccessEvent(pub String);

#[derive(Event)]
pub struct WorldSaveSuccessEvent(pub String);

// User callable events begin with "Request"

#[derive(Event)]
pub struct RequestSaveEvent(pub String);

#[derive(Event)]
pub struct RequestReloadEvent(pub String);

#[derive(Event)]
pub struct RequestLoadEvent(pub String);

#[derive(Event)]
pub struct RequestDespawnSerializableEntities;

#[derive(Event)]
pub struct RequestDespawnBySource(pub String);