use crate::RectBrush;

use super::{update_rectangle_brush_system, UserUpdatedRectBrushEvent};
use bevy::app::{App, Plugin, Update};

pub struct RectBrushPlugin;
impl Plugin for RectBrushPlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Event
            //
            .add_message::<UserUpdatedRectBrushEvent>()
            //
            // Register
            //
            .register_type::<RectBrush>()
            //
            // Schedule system
            //
            .add_systems(Update, update_rectangle_brush_system);
    }
}
