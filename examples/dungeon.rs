//dungeon.scene designed by Noah Booker
use bevy::prelude::*;
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
    let mut app = App::new();
    register_editor_components!();

    // Configure AssetPlugin for bundled assets when bundler feature is enabled
    #[cfg(feature = "bundler")]
    {
        println!("Dungeon example using bundler");
        use bevy_assets_bundler::{AssetBundlingOptions, BundledAssetIoPlugin};
        
        let options = AssetBundlingOptions::default();
        
        app.register_asset_source(
            bevy::asset::io::AssetSourceId::Default,
            BundledAssetIoPlugin::create_source_builder_or_default(options),
        );
    }

    app.add_plugins(DefaultPlugins.set(bevy::asset::AssetPlugin {
        meta_check: bevy::asset::AssetMetaCheck::Never,
        ..Default::default()
    }))
        .add_plugins(bevy_granite::BevyGranite {
            default_world: STARTING_WORLD.to_string(),
            ..Default::default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut open_event: MessageWriter<RequestLoadEvent>) {
    open_event.write(RequestLoadEvent(
        STARTING_WORLD.to_string(),
        SaveSettings::Runtime,
        None,
    ));
}
