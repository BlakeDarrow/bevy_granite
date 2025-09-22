use bevy::{prelude::Event, transform::components::Transform};
use crate::entities::SaveSettings;

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

/// Request the loading of serialized save data from a file. Optionally takes a Transform override to act as new loaded origin
#[derive(Event)]
pub struct RequestLoadEvent(pub String, pub SaveSettings, pub Option<Transform>);

#[derive(Event)]
pub struct RequestDespawnSerializableEntities;

#[derive(Event)]
pub struct RequestDespawnBySource(pub String);


