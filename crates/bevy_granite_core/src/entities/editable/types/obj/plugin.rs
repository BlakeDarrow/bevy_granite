use super::{update_obj_system, UserUpdatedOBJEvent};
use crate::OBJ;
use bevy::app::{App, Plugin, Update};

pub struct OBJPlugin;
impl Plugin for OBJPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Event
            //
            .add_message::<UserUpdatedOBJEvent>()
            //
            // Register
            //
            .register_type::<OBJ>()
            //
            // Schedule system
            //
            .add_systems(Update, update_obj_system);
    }
}
