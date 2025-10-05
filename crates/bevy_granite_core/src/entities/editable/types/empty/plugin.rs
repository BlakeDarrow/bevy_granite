use super::{update_empty_system, Empty, UserUpdatedEmptyEvent};
use bevy::app::{App, Plugin, Update};

pub struct EmptyPlugin;
impl Plugin for EmptyPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Event
            //
            .add_message::<UserUpdatedEmptyEvent>()
            //
            // Register
            //
            .register_type::<Empty>()
            //
            // Schedule system
            //
            .add_systems(Update, update_empty_system);
    }
}
