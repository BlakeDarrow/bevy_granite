use super::{update_camera_3d_system, UserUpdatedCamera3DEvent};
use crate::Camera3D;
use bevy::app::{App, Plugin, Update};

pub struct Camera3DPlugin;
impl Plugin for Camera3DPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Event
            //
            .add_message::<UserUpdatedCamera3DEvent>()
            //
            // Register
            //
            .register_type::<Camera3D>()
            //
            // Schedule system
            //
            .add_systems(Update, update_camera_3d_system);
    }
}
