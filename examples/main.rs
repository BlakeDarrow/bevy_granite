// main.rs - bevy 0.16
use bevy::prelude::*;
use bevy_granite::prelude::*;
use bevy_granite_core::entities::SaveSettings;

const STARTING_WORLD: &str = "scenes/starting.scene"; // Your starting world file. Doesn't have to actually exist yet

// Macro to define a component that can be edited in Granite
// This also adds the necessary derives for saving/loading
// You can add `default` to the argument to then add a 'Default' implementation
// You can also add `ui_hidden` to hide the component from the UI
// This component can be queried in bevy like any other component
#[granite_component]
struct MyTestComponent {
    value: i32,
}

#[granite_component("default")]
struct AnotherComponent {
    message: String,
}

impl Default for AnotherComponent {
    fn default() -> Self {
        AnotherComponent {
            message: "Hello, Granite!".to_string(),
        }
    }
}

fn main() {
    let mut app = App::new();

    // Critical macro to register all your editor components
    // If components are not registered here they will not be available in the editor
    // So you should import all your components in main.rs and then call this macro
    register_editor_components!();

    app.add_plugins(DefaultPlugins)
        // Add the Granite plugin
        // Make sure to pass a default world otherwise a default will be used
        // It it used to easily save/load your starting world via UI
        .add_plugins(bevy_granite::BevyGranite {
            default_world: STARTING_WORLD.to_string(),
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut open_event: EventWriter<RequestLoadEvent>) {
    // Event to load a world (.scene) with optional offset Transform
    // When finished loading it will send a `WorldLoadSuccessEvent` with the loaded world str name
    open_event.write(RequestLoadEvent(
        STARTING_WORLD.to_string(),
        SaveSettings::Runtime,
        None,
    ));
}
