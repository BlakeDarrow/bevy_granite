use crate::DirLight;

use super::{update_directional_light_system, UserUpdatedDirectionalLightEvent};
use bevy::app::{App, Plugin, Update};

pub struct DirLightPlugin;
impl Plugin for DirLightPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Event
            //
            .add_message::<UserUpdatedDirectionalLightEvent>()
            //
            // Register
            //
            .register_type::<DirLight>()
            //
            // Schedule system
            //
            .add_systems(Update, update_directional_light_system);
    }
}
