use crate::entities::SaveSettings;
use bevy::{ecs::message::Message, prelude::Event, transform::components::Transform};

#[derive(Message)]
pub struct RuntimeDataReadyEvent(pub String);

#[derive(Message)]
pub struct CollectRuntimeDataEvent(pub String);

#[derive(Message)]
pub struct WorldLoadSuccessEvent(pub String);

#[derive(Message)]
pub struct WorldSaveSuccessEvent(pub String);

// User callable events begin with "Request"

#[derive(Message)]
pub struct RequestSaveEvent(pub String);

#[derive(Message)]
pub struct RequestReloadEvent(pub String);

/// Request the loading of serialized save data from a file. Optionally takes a Transform override to act as new loaded origin
#[derive(Message)]
pub struct RequestLoadEvent(pub String, pub SaveSettings, pub Option<Transform>);

#[derive(Message)]
pub struct RequestDespawnSerializableEntities;

#[derive(Message)]
pub struct RequestDespawnBySource(pub String);
