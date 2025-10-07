//dungeon.scene designed by Noah Booker
use bevy::{
    prelude::*,
    remote::{http::RemoteHttpPlugin, RemotePlugin},
};
use bevy_granite::prelude::*;
use bevy_granite_core::entities::SaveSettings;

const STARTING_WORLD: &str = "scenes/dungeon.scene";

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
    let path = "./target/debug/examples/brp_editor.exe";
    let mut _editor = std::process::Command::new(path)
        .spawn()
        .expect("failed to start editor");
    let mut app = App::new();
    register_editor_components!();

    app.add_plugins(DefaultPlugins)
        .add_plugins(bevy_granite::BevyGranite {
            default_world: STARTING_WORLD.to_string(),
            ..Default::default()
        })
        .add_plugins((RemotePlugin::default(), RemoteHttpPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, close_if_editor_closed)
        .insert_resource(EditorHandle(_editor))
        .run();
    // _ = _editor.wait();
}

fn setup(mut open_event: MessageWriter<RequestLoadEvent>) {
    open_event.write(RequestLoadEvent(
        STARTING_WORLD.to_string(),
        SaveSettings::Runtime,
        None,
    ));
}

#[derive(Resource, Deref, DerefMut)]
struct EditorHandle(std::process::Child);

fn close_if_editor_closed(mut editor: ResMut<EditorHandle>, mut exit: MessageWriter<AppExit>) {
    if let Ok(Some(_)) = editor.try_wait() {
        println!("Editor closed, shutting down game.");
        exit.write(AppExit::default());
    }
}
