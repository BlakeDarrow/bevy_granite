use super::{update_spot_light_system, UserUpdatedSpotLightEvent, SpotLightData};
use bevy::app::{App, Plugin, Update};

pub struct SpotLightPlugin;
impl Plugin for SpotLightPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Event
            //
            .add_message::<UserUpdatedSpotLightEvent>()
            //
            // Register
            //
            .register_type::<SpotLightData>()
            //
            // Schedule system
            //
            .add_systems(Update, update_spot_light_system);
    }
}
