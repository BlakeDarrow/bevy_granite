use bevy::camera::visibility::RenderLayers;

pub const LAYER_SCENE: usize = 0;
pub const LAYER_GRID: usize = 13;
pub const LAYER_GIZMO: usize = 14;
pub const LAYER_UI: usize = 31;

// WASM/WebGL has different mouse wheel sensitivity, so we scale it
#[cfg(target_arch = "wasm32")]
pub const WASM_SCROLL_SENSITIVITY_MULTIPLIER: f32 = 0.05;

#[cfg(not(target_arch = "wasm32"))]
pub const WASM_SCROLL_SENSITIVITY_MULTIPLIER: f32 = 1.0;

pub fn scene_layers() -> RenderLayers {
    RenderLayers::from_layers(&[LAYER_SCENE, LAYER_GRID])
}

pub fn grid_layers() -> RenderLayers {
    RenderLayers::layer(LAYER_GRID)
}

pub fn gizmo_layers() -> RenderLayers {
    RenderLayers::layer(LAYER_GIZMO)
}

pub fn ui_layers() -> RenderLayers {
    RenderLayers::layer(LAYER_UI)
}
