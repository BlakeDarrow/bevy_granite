use crate::interface::popups::PopupType;
use crate::interface::tabs::entity_editor::{
    EntityGlobalTransformData, EntityIdentityData, EntityRegisteredData,
};
use bevy::ecs::event::EventWriter;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Event, Vec2};
use bevy_granite_core::RequestDespawnBySource;
use bevy_granite_core::RequestDespawnSerializableEntities;
use bevy_granite_core::{EditableMaterial, GraniteTypes};
use bevy_granite_core::{RequestLoadEvent, RequestReloadEvent, RequestSaveEvent};

#[derive(SystemParam)]
pub struct EditorEvents<'w> {
    pub popup: EventWriter<'w, PopupMenuRequestedEvent>,
    pub save: EventWriter<'w, RequestSaveEvent>,
    pub reload: EventWriter<'w, RequestReloadEvent>,
    pub load: EventWriter<'w, RequestLoadEvent>,
    pub toggle_editor: EventWriter<'w, RequestEditorToggle>,
    pub toggle_cam_sync: EventWriter<'w, RequestToggleCameraSync>,
    pub viewport_camera: EventWriter<'w, RequestViewportCameraOverride>,
    pub frame: EventWriter<'w, RequestCameraEntityFrame>,
    pub parent: EventWriter<'w, RequestNewParent>,
    pub remove_parent: EventWriter<'w, RequestRemoveParents>,
    pub remove_parent_entities: EventWriter<'w, RequestRemoveParentsFromEntities>,
    pub remove_children: EventWriter<'w, RequestRemoveChildren>,
    pub despawn_all: EventWriter<'w, RequestDespawnSerializableEntities>,
    pub despawn_by_source: EventWriter<'w, RequestDespawnBySource>,
    pub set_active_world: EventWriter<'w, SetActiveWorld>,
}

// Internal Events

#[derive(Event)]
pub struct UserUpdatedComponentsEvent {
    pub entity: Entity,
    pub data: EntityRegisteredData,
}

#[derive(Event)]
pub struct UserUpdatedIdentityEvent {
    pub entity: Entity,
    pub data: EntityIdentityData,
}

#[derive(Event)]
pub struct UserUpdatedTransformEvent {
    pub entity: Entity,
    pub data: EntityGlobalTransformData,
}

// Need to change this to the actual data struct instead. No need to have both structs
#[derive(Event)]
pub struct UserRequestGraniteTypeViaPopup {
    pub class: GraniteTypes,
}

#[derive(Event)]
pub struct UserRequestedRelationShipEvent;

#[derive(Event)]
pub struct SetActiveWorld(pub String);

#[derive(Event)]
pub struct PopupMenuRequestedEvent {
    pub popup: PopupType,
    pub mouse_pos: Vec2,
}

#[derive(Event)]
pub struct MaterialHandleUpdateEvent {
    pub skip_entity: Entity, // Requestor
    pub path: String,        // Path of updated EditableMaterial
    pub version: u32,
    pub material: EditableMaterial,
}

#[derive(Event)]
pub struct MaterialDeleteEvent {
    pub path: String,
}

// User callable events

#[derive(Event)]
pub struct RequestEditorToggle;

#[derive(Event)]
pub struct RequestCameraEntityFrame;

#[derive(Event)]
pub struct RequestToggleCameraSync;

#[derive(Event)]
pub struct RequestViewportCameraOverride {
    pub camera: Option<Entity>,
}

#[derive(Event)]
pub struct RequestNewParent;

#[derive(Event)]
pub struct RequestRemoveParents;

#[derive(Event)]
pub struct RequestRemoveParentsFromEntities {
    pub entities: Vec<Entity>,
}

#[derive(Event)]
pub struct RequestRemoveChildren;
