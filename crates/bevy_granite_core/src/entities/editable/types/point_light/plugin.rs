use super::{update_point_light_system, UserUpdatedPointLightEvent};
use crate::PointLightData;
use bevy::app::{App, Plugin, Update};

pub struct PointLightPlugin;
impl Plugin for PointLightPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Event
            //
            .add_message::<UserUpdatedPointLightEvent>()
            //
            // Register
            //
            .register_type::<PointLightData>()
            //
            // Schedule system
            //
            .add_systems(Update, update_point_light_system);
    }
}
