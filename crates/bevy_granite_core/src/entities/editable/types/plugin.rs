use super::*;
use bevy::app::{App, Plugin};
pub struct ClassTypePlugin;
impl Plugin for ClassTypePlugin {
    fn build(&self, app: &mut App) {
        app
            //
            // Class types
            .add_plugins(Camera3DPlugin)
            .add_plugins(DirLightPlugin)
            .add_plugins(PointLightPlugin)
            .add_plugins(SpotLightPlugin)
            .add_plugins(RectBrushPlugin)
            .add_plugins(EmptyPlugin)
            .add_plugins(OBJPlugin);
    }
}
