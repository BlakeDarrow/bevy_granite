use crate::{
    editor_state::INPUT_CONFIG,
    entities::bounds::get_entity_bounds_world,
    interface::events::{RequestCameraEntityFrame, RequestToggleCameraSync},
    viewport::camera::{handle_movement, handle_zoom, rotate_camera_towards},
};
use bevy::{
    asset::Assets,
    ecs::entity::Entity,
    input::mouse::{MouseMotion, MouseWheel},
    mesh::{Mesh, Mesh3d},
    prelude::{
        Local, MessageReader, Query, Res, ResMut, Resource, Time, Transform, Vec2, Vec3, Window,
        With, Without,
    },
    transform::components::GlobalTransform,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_granite_core::{MainCamera, UICamera, UserInput};
use bevy_granite_gizmos::{ActiveSelection, DragState, Selected};
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

pub fn sync_cameras_system(
    mut ui_camera_query: Query<&mut Transform, With<UICamera>>,
    mut main_camera_query: Query<&mut Transform, (With<MainCamera>, Without<UICamera>)>,
    mut camera_state: ResMut<CameraSyncState>,
) {
    // Who has control of camera
    if camera_state.ui_camera_has_control {
        if let Some(stored_ui_transform) = camera_state.ui_camera_old_position {
            if let Some(mut ui_camera_transform) = ui_camera_query.iter_mut().next() {
                ui_camera_transform.translation = stored_ui_transform.translation;
                ui_camera_transform.rotation = stored_ui_transform.rotation;

                camera_state.ui_camera_old_position = None;
            }
        } else {
            // UICamera has control
            if let Some(ui_camera_transform) = ui_camera_query.iter().next() {
                if let Some(mut main_camera_transform) = main_camera_query.iter_mut().next() {
                    main_camera_transform.translation = ui_camera_transform.translation;
                    main_camera_transform.rotation = ui_camera_transform.rotation;
                }
            }
        }
    } else {
        // MainCamera has control
        if let Some(main_camera_transform) = main_camera_query.iter().next() {
            if let Some(mut ui_camera_transform) = ui_camera_query.iter_mut().next() {
                ui_camera_transform.translation = main_camera_transform.translation;
                ui_camera_transform.rotation = main_camera_transform.rotation;
            }
        }
    }
}

// Whether or not we want control of main camera
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
    } else {
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
