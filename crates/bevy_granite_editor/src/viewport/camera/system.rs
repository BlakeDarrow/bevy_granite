use crate::{
    editor_state::{EditorState, INPUT_CONFIG},
    entities::bounds::get_entity_bounds_world,
    interface::events::{
        RequestCameraEntityFrame, RequestToggleCameraSync, RequestViewportCameraOverride,
    },
    viewport::camera::{
        handle_movement, handle_zoom, rotate_camera_towards, ViewportCameraState, LAYER_GIZMO,
        LAYER_GRID, LAYER_SCENE,
    },
};
use bevy::{
    asset::Assets,
    camera::{visibility::RenderLayers, Camera, Camera3d, RenderTarget, Viewport},
    ecs::{entity::Entity, system::Commands},
    input::mouse::{MouseMotion, MouseWheel},
    mesh::{Mesh, Mesh3d},
    prelude::{
        Local, MessageReader, Query, Res, ResMut, Resource, Time, Transform, UVec2, Vec2, Vec3,
        Window, With, Without,
    },
    transform::components::GlobalTransform,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_egui::EguiContexts;
use bevy_granite_core::{MainCamera, UICamera, UserInput};
use bevy_granite_gizmos::{
    ActiveSelection, DragState, GizmoCamera, GizmoVisibilityState, Selected,
};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

#[derive(Resource, Default)]
pub struct InputState {
    initial_cursor_pos: Option<Vec2>,
}

#[derive(Resource, Default)]
pub struct CameraTarget {
    pub position: Vec3,
}

#[derive(Resource)]
pub struct CameraSyncState {
    pub ui_camera_has_control: bool,
    pub ui_camera_old_position: Option<Transform>,
}

impl Default for CameraSyncState {
    fn default() -> Self {
        Self {
            ui_camera_has_control: true,
            ui_camera_old_position: None,
        }
    }
}

fn compute_viewport_layers(existing: Option<&RenderLayers>) -> RenderLayers {
    let mut layers = vec![LAYER_SCENE, LAYER_GRID];
    if let Some(existing_layers) = existing {
        for layer in existing_layers.iter() {
            if layer == LAYER_GIZMO {
                continue;
            }
            if !layers.contains(&layer) {
                layers.push(layer);
            }
        }
    }
    RenderLayers::from_layers(&layers)
}

fn restore_render_layers(commands: &mut Commands, entity: Entity, layers: Option<RenderLayers>) {
    if let Some(layers) = layers {
        commands.entity(entity).insert(layers);
    } else {
        commands.entity(entity).remove::<RenderLayers>();
    }
}

pub fn sync_cameras_system(
    mut commands: Commands,
    mut ui_camera_query: Query<&mut Transform, With<UICamera>>,
    mut active_camera_query: Query<
        &mut Transform,
        (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>),
    >,
    mut camera_state: ResMut<CameraSyncState>,
    mut viewport_camera_state: ResMut<ViewportCameraState>,
) {
    let Ok(mut ui_camera_transform) = ui_camera_query.single_mut() else {
        return;
    };

    let Some(active_camera_entity) = viewport_camera_state.active_camera() else {
        return;
    };

    let mut active_camera_transform = match active_camera_query.get_mut(active_camera_entity) {
        Ok(transform) => transform,
        Err(_) => {
            if viewport_camera_state.active_override.is_some() {
                if let Some((stored_entity, stored_layers)) =
                    viewport_camera_state.take_override_render_layers()
                {
                    restore_render_layers(&mut commands, stored_entity, stored_layers);
                }
                viewport_camera_state.clear_override();
                if let Some(stored_transform) = viewport_camera_state.take_stored_editor_transform()
                {
                    ui_camera_transform.translation = stored_transform.translation;
                    ui_camera_transform.rotation = stored_transform.rotation;

                    if let Some(editor_entity) = viewport_camera_state.editor_camera {
                        if let Ok(mut editor_transform) = active_camera_query.get_mut(editor_entity)
                        {
                            *editor_transform = stored_transform;
                        }
                    }
                }
            }
            return;
        }
    };

    if camera_state.ui_camera_has_control {
        if let Some(stored_ui_transform) = camera_state.ui_camera_old_position.take() {
            ui_camera_transform.translation = stored_ui_transform.translation;
            ui_camera_transform.rotation = stored_ui_transform.rotation;
        } else {
            active_camera_transform.translation = ui_camera_transform.translation;
            active_camera_transform.rotation = ui_camera_transform.rotation;
        }
    } else {
        ui_camera_transform.translation = active_camera_transform.translation;
        ui_camera_transform.rotation = active_camera_transform.rotation;
    }
}

// Whether or not we want control of the active viewport camera
pub fn camera_sync_toggle_system(
    mut toggle_event_writer: MessageReader<RequestToggleCameraSync>,
    mut sync: ResMut<CameraSyncState>,
    ui_camera_query: Query<&Transform, With<UICamera>>,
) {
    for _event in toggle_event_writer.read() {
        // Store UI camera position when disabling sync (before UICamera takes control)
        if sync.ui_camera_has_control {
            if let Ok(ui_camera_transform) = ui_camera_query.single() {
                sync.ui_camera_old_position = Some(*ui_camera_transform);
            }
        }

        log!(
            LogType::Editor,
            LogLevel::OK,
            LogCategory::System,
            "Toggled camera control sync"
        );
        sync.ui_camera_has_control = !sync.ui_camera_has_control;
    }
}

pub fn enforce_viewport_camera_state(
    viewport_camera_state: Res<ViewportCameraState>,
    mut camera_query: Query<
        (Entity, &mut Camera),
        (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>),
    >,
) {
    let Some(active_camera_entity) = viewport_camera_state.active_camera() else {
        return;
    };

    let mut active_found = false;

    for (entity, mut camera) in camera_query.iter_mut() {
        if entity == active_camera_entity {
            active_found = true;
            camera.is_active = true;
        } else if matches!(camera.target, RenderTarget::Window(_)) {
            camera.is_active = false;
        }
    }

    if !active_found {
        // The active camera is missing; the sync system will fall back to the editor camera.
    }
}

pub fn restore_runtime_camera_state(
    mut commands: Commands,
    mut viewport_camera_state: ResMut<ViewportCameraState>,
    mut ui_camera_query: Query<&mut Transform, With<UICamera>>,
    mut camera_transform_query: Query<
        &mut Transform,
        (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>),
    >,
    mut camera_query: Query<
        (Entity, &mut Camera),
        (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>),
    >,
    main_camera_entities: Query<Entity, With<MainCamera>>,
) {
    if let Some(active_override) = viewport_camera_state.active_override {
        if let Some((stored_entity, stored_layers)) =
            viewport_camera_state.take_override_render_layers()
        {
            if stored_entity == active_override {
                restore_render_layers(&mut commands, stored_entity, stored_layers);
            } else {
                viewport_camera_state.store_override_render_layers(stored_entity, stored_layers);
            }
        }

        if let Ok(mut ui_transform) = ui_camera_query.single_mut() {
            if let Some(stored_transform) = viewport_camera_state.take_stored_editor_transform() {
                ui_transform.translation = stored_transform.translation;
                ui_transform.rotation = stored_transform.rotation;

                if let Some(editor_entity) = viewport_camera_state.editor_camera {
                    if let Ok(mut editor_transform) = camera_transform_query.get_mut(editor_entity)
                    {
                        *editor_transform = stored_transform;
                    }
                }
            }
        }
        viewport_camera_state.clear_override();
    }

    let main_entities: Vec<Entity> = main_camera_entities.iter().collect();
    let mut any_main_enabled = false;

    for entity in &main_entities {
        if let Ok((_, mut camera)) = camera_query.get_mut(*entity) {
            if matches!(camera.target, RenderTarget::Window(_)) {
                camera.is_active = true;
                any_main_enabled = true;
            }
        }
    }

    if any_main_enabled {
        for (entity, mut camera) in camera_query.iter_mut() {
            if !main_entities.contains(&entity) && matches!(camera.target, RenderTarget::Window(_))
            {
                camera.is_active = false;
            }
        }
    } else {
        for (_, mut camera) in camera_query.iter_mut() {
            if matches!(camera.target, RenderTarget::Window(_)) {
                camera.is_active = true;
            }
        }
    }

    // Clear ALL custom viewports when exiting editor
    for (_, mut camera) in camera_query.iter_mut() {
        if matches!(camera.target, RenderTarget::Window(_)) {
            camera.viewport = None;
        }
    }
}

pub fn handle_viewport_camera_override_requests(
    mut requests: MessageReader<RequestViewportCameraOverride>,
    mut viewport_camera_state: ResMut<ViewportCameraState>,
    mut camera_sync_state: ResMut<CameraSyncState>,
    mut commands: Commands,
    mut ui_camera_query: Query<&mut Transform, With<UICamera>>,
    mut camera_transform_query: Query<
        &mut Transform,
        (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>),
    >,
    camera_meta_query: Query<&Camera, (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>)>,
    render_layers_query: Query<
        &RenderLayers,
        (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>),
    >,
) {
    for RequestViewportCameraOverride { camera } in requests.read() {
        let Ok(mut ui_transform) = ui_camera_query.single_mut() else {
            continue;
        };

        if let Some(target_entity) = camera {
            if Some(*target_entity) == viewport_camera_state.active_override {
                continue;
            }

            if let Some(current_override) = viewport_camera_state.active_override {
                if current_override != *target_entity {
                    if let Some((stored_entity, stored_layers)) =
                        viewport_camera_state.take_override_render_layers()
                    {
                        if stored_entity == current_override {
                            restore_render_layers(&mut commands, stored_entity, stored_layers);
                        } else {
                            viewport_camera_state
                                .store_override_render_layers(stored_entity, stored_layers);
                        }
                    }
                }
            }

            let Ok(target_camera) = camera_meta_query.get(*target_entity) else {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::System,
                    "Requested viewport camera {:?} is missing",
                    target_entity
                );
                continue;
            };

            if !matches!(target_camera.target, RenderTarget::Window(_)) {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::System,
                    "Requested viewport camera {:?} is not targeting a window and cannot take over",
                    target_entity
                );
                continue;
            }

            if viewport_camera_state.is_using_editor() {
                if let Some(editor_entity) = viewport_camera_state.editor_camera {
                    if let Ok(editor_transform) = camera_transform_query.get_mut(editor_entity) {
                        viewport_camera_state.store_editor_transform(editor_transform.clone());
                    }
                }
            }

            match camera_transform_query.get_mut(*target_entity) {
                Ok(target_transform) => {
                    ui_transform.translation = target_transform.translation;
                    ui_transform.rotation = target_transform.rotation;
                }
                Err(_) => {
                    log!(
                        LogType::Editor,
                        LogLevel::Warning,
                        LogCategory::System,
                        "Requested viewport camera {:?} has no transform",
                        target_entity
                    );
                    continue;
                }
            }

            let existing_layers = render_layers_query.get(*target_entity).ok().cloned();
            let new_layers = compute_viewport_layers(existing_layers.as_ref());
            viewport_camera_state.store_override_render_layers(*target_entity, existing_layers);
            commands.entity(*target_entity).insert(new_layers);

            viewport_camera_state.set_override(*target_entity);
            camera_sync_state.ui_camera_old_position = None;
        } else {
            if viewport_camera_state.active_override.is_none() {
                continue;
            }

            if let Some((stored_entity, stored_layers)) =
                viewport_camera_state.take_override_render_layers()
            {
                if let Some(active_override) = viewport_camera_state.active_override {
                    if stored_entity == active_override {
                        restore_render_layers(&mut commands, stored_entity, stored_layers);
                    } else {
                        viewport_camera_state
                            .store_override_render_layers(stored_entity, stored_layers);
                    }
                } else {
                    restore_render_layers(&mut commands, stored_entity, stored_layers);
                }
            }

            viewport_camera_state.clear_override();
            camera_sync_state.ui_camera_old_position = None;

            if let Some(stored_transform) = viewport_camera_state.take_stored_editor_transform() {
                ui_transform.translation = stored_transform.translation;
                ui_transform.rotation = stored_transform.rotation;

                if let Some(editor_entity) = viewport_camera_state.editor_camera {
                    if let Ok(mut editor_transform) = camera_transform_query.get_mut(editor_entity)
                    {
                        *editor_transform = stored_transform;
                    }
                }
            }
        }
    }
}

pub fn update_viewport_camera_viewports_system(
    mut contexts: EguiContexts,
    editor_state: Res<EditorState>,
    viewport_camera_state: Res<ViewportCameraState>,
    mut camera_query: Query<&mut Camera, (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>)>,
    mut gizmo_camera_query: Query<&mut Camera, With<GizmoCamera>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok(primary_window) = primary_window_query.single() else { return };

    let surface_width = primary_window.resolution.physical_width() as f32;
    let surface_height = primary_window.resolution.physical_height() as f32;
    if surface_width < 1.0 || surface_height < 1.0 {
        return;
    }

    let mut clear_viewports = || {
        if let Ok(mut gizmo) = gizmo_camera_query.single_mut() {
            gizmo.viewport = None;
        }
    };

    let Some(active_camera) = viewport_camera_state.active_camera() else {
        clear_viewports();
        return;
    };

    if !editor_state.active {
        if let Ok(mut camera) = camera_query.get_mut(active_camera) {
            camera.viewport = None;
        }
        clear_viewports();
        return;
    }

    let available_rect = ctx.available_rect();
    if available_rect.width() <= 1.0 || available_rect.height() <= 1.0 {
        if let Ok(mut camera) = camera_query.get_mut(active_camera) {
            camera.viewport = None;
        }
        clear_viewports();
        return;
    }

    // Clamp the egui rect against the full window rect so we never request a viewport outside the surface.
    let scale = ctx.pixels_per_point();
    let mut min_px = (available_rect.min * scale).floor();
    let mut max_px = (available_rect.max * scale).ceil();

    min_px.x = min_px.x.clamp(0.0, surface_width);
    min_px.y = min_px.y.clamp(0.0, surface_height);
    max_px.x = max_px.x.clamp(min_px.x, surface_width);
    max_px.y = max_px.y.clamp(min_px.y, surface_height);

    if max_px.x - min_px.x < 1.0 || max_px.y - min_px.y < 1.0 {
        if let Ok(mut camera) = camera_query.get_mut(active_camera) {
            camera.viewport = None;
        }
        clear_viewports();
        return;
    }

    let viewport = Viewport {
        physical_position: UVec2::new(min_px.x as u32, min_px.y as u32),
        physical_size: UVec2::new((max_px.x - min_px.x) as u32, (max_px.y - min_px.y) as u32),
        ..Default::default()
    };

    if viewport.physical_size.x == 0 || viewport.physical_size.y == 0 {
        if let Ok(mut camera) = camera_query.get_mut(active_camera) {
            camera.viewport = None;
        }
        clear_viewports();
        return;
    }

    if let Ok(mut camera) = camera_query.get_mut(active_camera) {
        camera.viewport = Some(viewport.clone());
    }
    if let Ok(mut gizmo_camera) = gizmo_camera_query.single_mut() {
        gizmo_camera.viewport = Some(viewport);
    }
}

pub fn sync_gizmo_camera_state(
    viewport_camera_state: Res<ViewportCameraState>,
    editor_state: Res<EditorState>,
    gizmo_visibility: Res<GizmoVisibilityState>,
    mut gizmo_camera_query: Query<(&mut Camera, &mut Transform), With<GizmoCamera>>,
    active_camera_query: Query<
        &Transform,
        (With<Camera3d>, Without<UICamera>, Without<GizmoCamera>),
    >,
) {
    let Ok((mut gizmo_camera, mut gizmo_transform)) = gizmo_camera_query.single_mut() else {
        return;
    };

    let should_render = editor_state.active && gizmo_visibility.active;
    gizmo_camera.is_active = should_render;

    if !should_render {
        gizmo_camera.viewport = None;
        return;
    }

    let Some(active_entity) = viewport_camera_state.active_camera() else {
        gizmo_camera.is_active = false;
        gizmo_camera.viewport = None;
        return;
    };

    if let Ok(active_transform) = active_camera_query.get(active_entity) {
        *gizmo_transform = active_transform.clone();
    }
}

pub fn camera_frame_system(
    transform_query: Query<&GlobalTransform, Without<UICamera>>,
    mut camera_query: Query<&mut Transform, With<UICamera>>,
    mut camera_target: ResMut<CameraTarget>,
    mut frame_reader: MessageReader<RequestCameraEntityFrame>,
    _user_input: Res<UserInput>,
    selected_query: Query<Entity, With<Selected>>,
    active_query: Query<Entity, With<ActiveSelection>>,
    meshes: Res<Assets<Mesh>>,
    mesh_query: Query<&Mesh3d>, // Needed for bounds
) {
    let frame_whole_selection = true;
    let base_distance: f32 = 10.;
    let distance_factor: f32 = 2.0; // Multiplier for bounding sphere radius
    let max_factor: f32 = 3.5; // Max distance is size * max_factor

    let camera_frame_exponent: f32 = 0.95;
    let camera_frame_pitch_deg: f32 = 35.0;
    let camera_frame_pitch_rad = camera_frame_pitch_deg.to_radians();
    let margin: f32 = 1.35; // 20% extra space
    for _ in frame_reader.read() {
        let selected_count = selected_query.iter().count();
        if frame_whole_selection && selected_count > 1 {
            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            let mut found = false;
            for entity in selected_query.iter() {
                if let Ok(global_transform) = transform_query.get(entity) {
                    if let Some((entity_min, entity_max)) =
                        get_entity_bounds_world(entity, &meshes, &mesh_query, global_transform)
                    {
                        min = min.min(entity_min);
                        max = max.max(entity_max);
                        found = true;
                    }
                }
            }
            if found {
                let center = (min + max) * 0.5;
                let radius = 0.5 * (max - min).length(); // Use bounding sphere radius
                let mut distance =
                    (radius.powf(camera_frame_exponent) * distance_factor).max(base_distance);
                let max_distance = radius * max_factor;
                distance = distance.min(max_distance);
                distance *= margin; // Add margin
                camera_target.position = center;
                for mut camera_transform in camera_query.iter_mut() {
                    let rel = camera_transform.translation - center;
                    let yaw = rel.z.atan2(rel.x);
                    let dir_x = camera_frame_pitch_rad.cos() * yaw.cos();
                    let dir_y = camera_frame_pitch_rad.sin();
                    let dir_z = camera_frame_pitch_rad.cos() * yaw.sin();
                    let final_direction = Vec3::new(dir_x, dir_y, dir_z).normalize();
                    camera_transform.translation = center + final_direction * distance;
                    rotate_camera_towards(&mut camera_transform, center, 1.0);
                }
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::System,
                    "Framing whole selection bounds"
                );
                return;
            }
        } else if selected_count == 1 {
            // Frame the single selected entity's bounds if possible
            let entity = selected_query.iter().next().unwrap();
            if let Ok(global_transform) = transform_query.get(entity) {
                if let Some((entity_min, entity_max)) =
                    get_entity_bounds_world(entity, &meshes, &mesh_query, global_transform)
                {
                    let center = (entity_min + entity_max) * 0.5;
                    let radius = 0.5 * (entity_max - entity_min).length();
                    let mut distance =
                        (radius.powf(camera_frame_exponent) * distance_factor).max(base_distance);
                    let max_distance = radius * max_factor;
                    distance = distance.min(max_distance);
                    distance *= margin;
                    camera_target.position = center;
                    for mut camera_transform in camera_query.iter_mut() {
                        let rel = camera_transform.translation - center;
                        let yaw = rel.z.atan2(rel.x);
                        let dir_x = camera_frame_pitch_rad.cos() * yaw.cos();
                        let dir_y = camera_frame_pitch_rad.sin();
                        let dir_z = camera_frame_pitch_rad.cos() * yaw.sin();
                        let final_direction = Vec3::new(dir_x, dir_y, dir_z).normalize();
                        camera_transform.translation = center + final_direction * distance;
                        rotate_camera_towards(&mut camera_transform, center, 1.0);
                    }
                    log!(
                        LogType::Editor,
                        LogLevel::Info,
                        LogCategory::System,
                        "Framing single selection bounds"
                    );
                    return;
                }
            }
            // If no bounds, fall through to default (origin) framing
        }

        // Default: frame active selection origin (fallback for entities without bounds)
        if selected_count > 0 {
            let entity = active_query.iter().next().unwrap();
            if let Ok(target_transform) = transform_query.get(entity) {
                camera_target.position = target_transform.translation();
                for mut camera_transform in camera_query.iter_mut() {
                    let rel = camera_transform.translation - camera_target.position;
                    let yaw = rel.z.atan2(rel.x);
                    let dir_x = camera_frame_pitch_rad.cos() * yaw.cos();
                    let dir_y = camera_frame_pitch_rad.sin();
                    let dir_z = camera_frame_pitch_rad.cos() * yaw.sin();
                    let final_direction = Vec3::new(dir_x, dir_y, dir_z).normalize();
                    camera_transform.translation =
                        camera_target.position + final_direction * base_distance;
                    rotate_camera_towards(&mut camera_transform, camera_target.position, 1.0);
                }
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::System,
                    "Framing selected entity origin"
                );
            } else {
                log!(
                    LogType::Editor,
                    LogLevel::Warning,
                    LogCategory::System,
                    "Selected entity has no transform to frame!"
                );
            }
        } else {
            log!(
                LogType::Editor,
                LogLevel::Warning,
                LogCategory::System,
                "No entity selected to frame!"
            );
        }
    }
}

// FIX:
// use new UserInput
pub fn mouse_button_iter(
    mut primary_window: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut query: Query<&mut Transform, With<UICamera>>,
    mut input_state: ResMut<InputState>,
    time: Res<Time>,
    mut target_pos: ResMut<CameraTarget>,
    user_input: Res<UserInput>,
    movement_speed: Local<f32>,
    drag_state: Res<DragState>,
) {
    if user_input.mouse_over_egui || drag_state.dragging {
        return;
    }

    if let Ok((mut window, mut cursor_options)) = primary_window.single_mut() {
        if user_input.mouse_right.just_pressed {
            cursor_options.visible = false;
            cursor_options.grab_mode = CursorGrabMode::Locked;
            input_state.initial_cursor_pos = window.cursor_position();
        }

        if user_input.mouse_right.just_released {
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
            if let Some(pos) = input_state.initial_cursor_pos {
                window.set_cursor_position(Some(pos));
            }
        }
    }

    if user_input.mouse_middle.pressed {
        handle_pan_or_rotation(
            &mut query,
            &user_input,
            &mut mouse_motion_events,
            &mut target_pos,
            time.delta_secs(),
        );
    }

    if user_input.mouse_right.pressed {
        handle_movement(
            &mut query,
            &user_input,
            &mut mouse_motion_events,
            &mut mouse_wheel_events,
            &mut target_pos,
            time,
            movement_speed,
        );
    } else if !user_input.mouse_middle.pressed {
        // Only handle zoom when not in FPS mode (right mouse) and not panning (middle mouse)
        handle_zoom(&mut query, &mut mouse_wheel_events, &mut target_pos);
    }
}

// Pan and Orbit
fn handle_pan_or_rotation(
    query: &mut Query<&mut Transform, With<UICamera>>,
    user_input: &Res<UserInput>,
    mouse_motion_events: &mut MessageReader<MouseMotion>,
    target_pos: &mut ResMut<CameraTarget>,
    delta_time: f32,
) {
    let pan_sensitivity = INPUT_CONFIG.pan_camera_sensitivity * delta_time;
    let rotate_sensitivity = INPUT_CONFIG.obit_camera_sensitivity * delta_time;
    let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.1;

    for mut camera_transform in query.iter_mut() {
        // Accumulate all mouse motion for this frame
        let mut accumulated_delta = Vec2::ZERO;
        for event in mouse_motion_events.read() {
            accumulated_delta += event.delta;
        }

        if accumulated_delta.length_squared() > 0.0 {
            if user_input.shift_left.pressed {
                let right = camera_transform.right() * -accumulated_delta.x * pan_sensitivity;
                let up = camera_transform.up() * accumulated_delta.y * pan_sensitivity;

                target_pos.position += right + up;
                camera_transform.translation += right + up;
            } else {
                let mut offset = camera_transform.translation - target_pos.position;
                let radius = offset.length();

                let mut spherical_pitch =
                    offset.y.atan2((offset.x.powi(2) + offset.z.powi(2)).sqrt());
                let mut spherical_yaw = offset.z.atan2(offset.x);

                spherical_yaw += accumulated_delta.x * rotate_sensitivity;
                spherical_pitch += accumulated_delta.y * rotate_sensitivity;
                spherical_pitch = spherical_pitch.clamp(-pitch_limit, pitch_limit);

                offset.x = radius * spherical_pitch.cos() * spherical_yaw.cos();
                offset.y = radius * spherical_pitch.sin();
                offset.z = radius * spherical_pitch.cos() * spherical_yaw.sin();

                camera_transform.translation = target_pos.position + offset;
                camera_transform.rotation = camera_transform
                    .looking_at(target_pos.position, Vec3::Y)
                    .rotation;
            }
        }
    }
}
