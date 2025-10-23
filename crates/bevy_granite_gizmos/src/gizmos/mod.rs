use bevy::{
    camera::visibility::RenderLayers,
    ecs::{component::Component, entity::Entity, lifecycle::HookContext, resource::Resource},
    prelude::{Deref, DerefMut},
};

pub mod distance_scaling;
pub mod events;
pub mod manager;
pub mod plugin;
pub mod rotate;
pub mod transform;
pub mod vertex;

#[derive(Clone, Default, Debug, Copy, PartialEq)]
pub enum GizmoType {
    Transform,
    Rotate,
    #[default]
    Pointer,
    None,
}

#[derive(Resource, Deref, DerefMut, Clone, Copy)]
pub struct NewGizmoType(pub GizmoType);

#[derive(Clone, Default, Debug, Copy, PartialEq)]
pub enum GizmoMode {
    Local,
    #[default]
    Global,
}

#[derive(Resource)]
pub struct NewGizmoConfig {
    pub speed_scale: f32,
    pub distance_scale: f32,
    pub mode: GizmoMode,
}

impl NewGizmoConfig {
    pub fn rotation(&self) -> GizmoConfig {
        GizmoConfig::Rotate {
            speed_scale: self.speed_scale,
            distance_scale: self.distance_scale,
            mode: self.mode,
        }
    }
    pub fn transform(&self) -> GizmoConfig {
        GizmoConfig::Transform {
            distance_scale: self.distance_scale,
            mode: self.mode,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum GizmoConfig {
    Pointer,
    None,
    Rotate {
        speed_scale: f32,
        distance_scale: f32,
        mode: GizmoMode,
    },
    Transform {
        distance_scale: f32,
        mode: GizmoMode,
    },
}

impl GizmoConfig {
    pub fn gizmo_type(&self) -> GizmoType {
        match self {
            GizmoConfig::Pointer => GizmoType::Pointer,
            GizmoConfig::None => GizmoType::None,
            GizmoConfig::Rotate { .. } => GizmoType::Rotate,
            GizmoConfig::Transform { .. } => GizmoType::Transform,
        }
    }

    pub fn mode(&self) -> GizmoMode {
        match self {
            GizmoConfig::Pointer => GizmoMode::Global,
            GizmoConfig::None => GizmoMode::Global,
            GizmoConfig::Rotate { mode, .. } => *mode,
            GizmoConfig::Transform { mode, .. } => *mode,
        }
    }

    pub fn set_type(&mut self, new_type: GizmoType, default_config: &NewGizmoConfig) {
        *self = match new_type {
            GizmoType::Pointer => GizmoConfig::Pointer,
            GizmoType::None => GizmoConfig::None,
            GizmoType::Rotate => GizmoConfig::Rotate {
                speed_scale: default_config.speed_scale,
                distance_scale: default_config.distance_scale,
                mode: default_config.mode,
            },
            GizmoType::Transform => GizmoConfig::Transform {
                distance_scale: default_config.distance_scale,
                mode: default_config.mode,
            },
        }
    }

    pub fn set_mode(&mut self, new_mode: GizmoMode) {
        match self {
            GizmoConfig::Pointer => {}
            GizmoConfig::None => {}
            GizmoConfig::Rotate { ref mut mode, .. } => {
                *mode = new_mode;
            }
            GizmoConfig::Transform { ref mut mode, .. } => {
                *mode = new_mode;
            }
        }
    }
}

#[derive(Resource)]
pub struct LastSelectedGizmo {
    pub value: GizmoType,
}

#[derive(Component)]
pub struct GizmoMesh;

#[derive(Component)]
#[relationship_target(relationship = GizmoRoot)]
pub struct GizmoChildren(Vec<Entity>);

#[derive(Resource)]
pub struct GizmoSnap {
    pub rotate_value: f32,
    pub transform_value: f32,
}

#[derive(Component, Deref, Clone, Copy)]
#[relationship(relationship_target = Gizmos)]
#[component(on_add = Self::on_add)]
#[require(bevy_granite_core::EditorIgnore, RenderLayers = RenderLayers::layer(14))]
pub struct GizmoOf(pub Entity);

#[derive(Component)]
#[relationship(relationship_target = GizmoChildren)]
pub struct GizmoRoot(pub Entity);

impl GizmoOf {
    fn on_add(mut world: bevy::ecs::world::DeferredWorld, ctx: HookContext) {
        let mut ignore = world
            .get_mut::<EditorIgnore>(ctx.entity)
            .expect("EditorIgnore is required component");
        ignore.insert(EditorIgnore::GIZMO | EditorIgnore::PICKING);
    }

    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Component)]
#[relationship_target(relationship = GizmoOf)]
pub struct Gizmos(Vec<Entity>);

impl Gizmos {
    pub fn entities(&self) -> &[Entity] {
        &self.0
    }
}

use bevy_granite_core::EditorIgnore;
pub use distance_scaling::scale_gizmo_by_camera_distance_system;
pub use events::{
    DespawnGizmoEvent, RotateDraggingEvent, RotateInitDragEvent, RotateResetDragEvent,
    SpawnGizmoEvent, TransformDraggingEvent, TransformInitDragEvent, TransformResetDragEvent,
};
pub use manager::{gizmo_changed_watcher, gizmo_events};
pub use plugin::GizmoPlugin;
pub use rotate::{
    despawn_rotate_gizmo, handle_init_rotate_drag, handle_rotate_dragging, handle_rotate_input,
    handle_rotate_reset, register_embedded_rotate_gizmo_mesh, spawn_rotate_gizmo, 
    update_gizmo_rotation_for_mode as update_rotate_gizmo_rotation_for_mode,
    RotateGizmo, RotateGizmoParent,
};
pub use transform::{
    despawn_transform_gizmo, spawn_transform_gizmo, update_gizmo_rotation_for_mode as update_transform_gizmo_rotation_for_mode, 
    PreviousTransformGizmo, TransformGizmo, TransformGizmoParent,
};
