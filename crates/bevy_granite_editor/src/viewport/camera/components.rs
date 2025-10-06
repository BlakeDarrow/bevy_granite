use bevy::prelude::{Component, Entity, Resource, Transform};

/// Marker component for the editor's dedicated 3D viewport camera.
#[derive(Component)]
pub struct EditorViewportCamera;

/// Tracks which camera should currently be rendered in the editor viewport.
#[derive(Resource, Default)]
pub struct ViewportCameraState {
    pub editor_camera: Option<Entity>,
    pub active_override: Option<Entity>,
    pub stored_editor_transform: Option<Transform>,
}

impl ViewportCameraState {
    pub fn active_camera(&self) -> Option<Entity> {
        self.active_override.or(self.editor_camera)
    }

    pub fn is_using_editor(&self) -> bool {
        self.active_override.is_none()
    }

    pub fn set_editor_camera(&mut self, entity: Entity) {
        self.editor_camera = Some(entity);
    }

    pub fn set_override(&mut self, entity: Entity) {
        self.active_override = Some(entity);
    }

    pub fn clear_override(&mut self) {
        self.active_override = None;
    }

    pub fn store_editor_transform(&mut self, transform: Transform) {
        self.stored_editor_transform = Some(transform);
    }

    pub fn take_stored_editor_transform(&mut self) -> Option<Transform> {
        self.stored_editor_transform.take()
    }
}

