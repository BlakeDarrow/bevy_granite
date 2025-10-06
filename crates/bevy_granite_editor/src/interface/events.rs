use crate::interface::popups::PopupType;
use crate::interface::tabs::entity_editor::{
    EntityGlobalTransformData, EntityIdentityData, EntityRegisteredData,
};
use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Message, Vec2};
use bevy_granite_core::RequestDespawnBySource;
use bevy_granite_core::RequestDespawnSerializableEntities;
use bevy_granite_core::{EditableMaterial, GraniteTypes};
use bevy_granite_core::{RequestLoadEvent, RequestReloadEvent, RequestSaveEvent};

#[derive(SystemParam)]
pub struct EditorEvents<'w> {
    pub popup: MessageWriter<'w, PopupMenuRequestedEvent>,
    pub save: MessageWriter<'w, RequestSaveEvent>,
    pub reload: MessageWriter<'w, RequestReloadEvent>,
    pub load: MessageWriter<'w, RequestLoadEvent>,
    pub toggle_editor: MessageWriter<'w, RequestEditorToggle>,
    pub toggle_cam_sync: MessageWriter<'w, RequestToggleCameraSync>,
    pub viewport_camera: MessageWriter<'w, RequestViewportCameraOverride>, // From #78
    pub frame: MessageWriter<'w, RequestCameraEntityFrame>,
    pub parent: MessageWriter<'w, RequestNewParent>,
    pub remove_parent: MessageWriter<'w, RequestRemoveParents>,
    pub remove_parent_entities: MessageWriter<'w, RequestRemoveParentsFromEntities>,
    pub remove_children: MessageWriter<'w, RequestRemoveChildren>,
    pub despawn_all: MessageWriter<'w, RequestDespawnSerializableEntities>,
    pub despawn_by_source: MessageWriter<'w, RequestDespawnBySource>,
    pub set_active_world: MessageWriter<'w, SetActiveWorld>,
}

// Internal Events

#[derive(Message)]
pub struct UserUpdatedComponentsEvent {
    pub entity: Entity,
    pub data: EntityRegisteredData,
}

#[derive(Message)]
pub struct UserUpdatedIdentityEvent {
    pub entity: Entity,
    pub data: EntityIdentityData,
}

#[derive(Message)]
pub struct UserUpdatedTransformEvent {
    pub entity: Entity,
    pub data: EntityGlobalTransformData,
}

// Need to change this to the actual data struct instead. No need to have both structs
#[derive(Message)]
pub struct UserRequestGraniteTypeViaPopup {
    pub class: GraniteTypes,
}

#[derive(Message)]
pub struct UserRequestedRelationShipEvent;

#[derive(Message)]
pub struct SetActiveWorld(pub String);

#[derive(Message)]
pub struct PopupMenuRequestedEvent {
    pub popup: PopupType,
    pub mouse_pos: Vec2,
}

#[derive(Message)]
pub struct MaterialHandleUpdateEvent {
    pub skip_entity: Entity, // Requestor
    pub path: String,        // Path of updated EditableMaterial
    pub version: u32,
    pub material: EditableMaterial,
}

#[derive(Message)]
pub struct MaterialDeleteEvent {
    pub path: String,
}

// User callable events

#[derive(Message)]
pub struct RequestEditorToggle;

#[derive(Message)]
pub struct RequestCameraEntityFrame;

#[derive(Message)]
pub struct RequestToggleCameraSync;

#[derive(Message)]
pub struct RequestViewportCameraOverride {
    pub camera: Option<Entity>,
}

#[derive(Message)]
pub struct RequestNewParent;

#[derive(Message)]
pub struct RequestRemoveParents;

#[derive(Message)]
pub struct RequestRemoveParentsFromEntities {
    pub entities: Vec<Entity>,
}

#[derive(Message)]
pub struct RequestRemoveChildren;
