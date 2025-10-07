use std::any::type_name;

use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    remote::{
        builtin_methods::{
            BrpMutateComponentsParams, BrpQuery, BrpQueryFilter, BrpQueryParams,
            BRP_MUTATE_COMPONENTS_METHOD, BRP_QUERY_METHOD,
        },
        BrpRequest,
    },
    window::{PrimaryWindow, WindowResolution},
};

const URL: &str = "http://localhost:15702";

fn main() {
    let mut app = App::new();
    // need this to run not on the main thread
    // this wont be a problem in a real use case since it would be its own executable
    app.add_plugins(DefaultPlugins);

    app.add_systems(Startup, hyjack_main_game_window);
    app.add_systems(PreStartup, spawn_fake_editor_ui);
    app.insert_resource(Time::<Fixed>::from_hz(5.));
    app.add_systems(FixedUpdate, sync_window_to_view);
    // app.add_systems(PostStartup, test_editor_hide_decorations);
    app.run();
}

// this fuction will get the entity of the main game window using BRP then it will hide the window decorations
fn hyjack_main_game_window(mut commands: Commands) {
    println!("Hyjacking main game window to remove decorations");
    let window = get_game_window(URL).expect("IDK");
    hide_decorations(URL, window).expect("IDK");
    commands.insert_resource(GameWindow(window));
}

fn test_editor_hide_decorations(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    let reflect = window.as_reflect_mut();
    let p = reflect.reflect_path_mut("decorations").unwrap();
    *p.try_downcast_mut::<bool>().unwrap() = false;
}

fn spawn_fake_editor_ui(mut commands: Commands) {
    println!("Spawning fake editor ui");
    commands.spawn((Camera2d, Tonemapping::None));
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::linear_rgb(1., 0.1, 0.1)),
        children![
            (
                Name::new("Fake Top Bar"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(30.),
                    top: Val::Px(0.),
                    ..Default::default()
                },
                Text::new("Fake Top Bar"),
                BackgroundColor(Color::linear_rgb(1., 0., 0.)),
            ),
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Row,
                    flex_grow: 1.0,
                    ..Default::default()
                },
                children![
                    (
                        Viewport,
                        Name::new("Fake Left Panel"),
                        Node {
                            width: Val::Percent(75.0),
                            height: Val::Auto,
                            left: Val::Px(0.),
                            ..Default::default()
                        },
                        Text::new("Fake Left Panel"),
                        BackgroundColor(Color::linear_rgb(1., 1., 1.)),
                    ),
                    (
                        Name::new("Fake Side Panel"),
                        Node {
                            width: Val::Percent(25.0),
                            height: Val::Auto,
                            right: Val::Px(0.),
                            ..Default::default()
                        },
                        Text::new("Fake Side Panel"),
                        BackgroundColor(Color::linear_rgb(0., 1., 0.)),
                    ),
                ]
            ),
            (
                Name::new("Fake Bottom Bar"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(100.),
                    bottom: Val::Px(0.),
                    ..Default::default()
                },
                Text::new("Fake Bottom Bar"),
                BackgroundColor(Color::linear_rgb(0., 0., 1.)),
            ),
        ],
    ));
    println!("Fake editor ui spawned");
}

#[derive(Component)]
struct Viewport;

#[derive(Resource)]
struct GameWindow(Entity);

fn get_game_window(url: &str) -> Result<Entity, anyhow::Error> {
    let get_transform_request = BrpRequest {
        jsonrpc: String::from("2.0"),
        method: String::from(BRP_QUERY_METHOD),
        id: Some(serde_json::to_value(1)?),
        params: Some(
            serde_json::to_value(BrpQueryParams {
                data: BrpQuery {
                    components: vec![],
                    ..Default::default()
                },
                strict: false,
                filter: BrpQueryFilter {
                    with: vec![type_name::<Window>().to_string()],
                    ..Default::default()
                },
            })
            .expect("Unable to convert query parameters to a valid JSON value"),
        ),
    };
    // bevy::prelude::info!("transform request: {get_transform_request:#?}");
    let res = ureq::post(url)
        .send_json(get_transform_request)?
        .body_mut()
        .read_json::<serde_json::Value>()?;
    let e = serde_json::from_value(res["result"][0]["entity"].clone())?;
    println!("game window entity: {e:?}");
    Ok(e)
}

fn hide_decorations(url: &str, window: Entity) -> Result<(), anyhow::Error> {
    let set_no_decorations_request = BrpRequest {
        jsonrpc: String::from("2.0"),
        method: String::from(BRP_MUTATE_COMPONENTS_METHOD),
        id: Some(serde_json::to_value(1)?),
        params: Some(serde_json::to_value(BrpMutateComponentsParams {
            entity: window,
            component: String::from(type_name::<Window>()),
            path: String::from("decorations"),
            value: serde_json::to_value(false)?,
        })?),
    };
    let res = ureq::post(url)
        .send_json(set_no_decorations_request)?
        .body_mut()
        .read_json::<serde_json::Value>()?;
    println!("{res:#}");
    Ok(())
}

fn sync_window_to_view(
    viewport: Single<(&ComputedNode, &UiTransform), With<Viewport>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    game_window: Res<GameWindow>,
) {
    if window.position == WindowPosition::Automatic {
        window.position = WindowPosition::At(IVec2::ZERO);
    }
    let (node, transform) = viewport.into_inner();
    println!("{} {:?} {:?}", node.size(), window.position, transform);
    send_sync_command(node.size().as_uvec2(), window.position, URL, game_window.0).expect("IDK");
}

fn send_sync_command(
    size: UVec2,
    position: WindowPosition,
    url: &str,
    window: Entity,
) -> Result<(), anyhow::Error> {
    let WindowPosition::At(mut pos) = position else {
        return Ok(());
    };
    pos.y += 30 + 30; // window decorations + fake top bar

    let set_no_decorations_request = BrpRequest {
        jsonrpc: String::from("2.0"),
        method: String::from(BRP_MUTATE_COMPONENTS_METHOD),
        id: Some(serde_json::to_value(1)?),
        params: Some(serde_json::to_value(BrpMutateComponentsParams {
            entity: window,
            component: String::from(type_name::<Window>()),
            path: String::from("position"),
            value: serde_json::to_value(WindowPosition::At(pos))?,
        })?),
    };
    let res = ureq::post(url)
        .send_json(set_no_decorations_request)?
        .body_mut()
        .read_json::<serde_json::Value>()?;

    println!("sync position res: {res:#?}");

    let set_no_decorations_request = BrpRequest {
        jsonrpc: String::from("2.0"),
        method: String::from(BRP_MUTATE_COMPONENTS_METHOD),
        id: Some(serde_json::to_value(1)?),
        params: Some(serde_json::to_value(BrpMutateComponentsParams {
            entity: window,
            component: String::from(type_name::<Window>()),
            path: String::from("resolution"),
            value: serde_json::to_value(WindowResolution::new(size.x, size.y))?,
        })?),
    };
    let res = ureq::post(url)
        .send_json(set_no_decorations_request)?
        .body_mut()
        .read_json::<serde_json::Value>()?;

    println!("sync resolution res: {res:#?}");
    let set_no_decorations_request = BrpRequest {
        jsonrpc: String::from("2.0"),
        method: String::from(BRP_MUTATE_COMPONENTS_METHOD),
        id: Some(serde_json::to_value(1)?),
        params: Some(serde_json::to_value(BrpMutateComponentsParams {
            entity: window,
            component: String::from(type_name::<Window>()),
            path: String::from("focused"),
            value: serde_json::to_value(true)?,
        })?),
    };
    let res = ureq::post(url)
        .send_json(set_no_decorations_request)?
        .body_mut()
        .read_json::<serde_json::Value>()?;
    println!("set focused res: {res:#?}");
    Ok(())
}
