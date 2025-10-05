// Apply the SAME world rotation delta to ROOT ENTITIES ONLY
// Children inherit rotation automatically through hierarchy
use crate::{
    gizmos::{
        GizmoConfig, GizmoMesh, GizmoOf, GizmoSnap, GizmoType, NewGizmoConfig, NewGizmoType,
        RotateDraggingEvent, RotateGizmo, RotateGizmoParent, RotateInitDragEvent,
        RotateResetDragEvent,
    },
    input::{DragState, GizmoAxis},
    selection::{
        ray::{raycast_at_cursor, HitType, RaycastCursorPos},
        ActiveSelection, RequestDuplicateAllSelectionEvent, Selected,
    },
    GizmoCamera,
};
use bevy::{
    camera::Camera,
    ecs::{observer::On, query::Changed, system::Local},
    math::primitives::InfinitePlane3d,
    picking::{
        events::{Drag, Pointer, Press},
        hover::PickingInteraction,
        pointer::PointerButton,
    },
    prelude::{
        ChildOf, Entity, GlobalTransform, MessageReader, MessageWriter, Mut, Name, ParamSet, Quat,
        Query, Res, ResMut, Transform, Vec2, Vec3, Visibility, With, Without,
    },
};
use bevy_granite_core::{CursorWindowPos, IconProxy, UserInput};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

// ------------------------------------------------------------------------
//
type ActiveSelectionQuery<'w, 's> = Query<'w, 's, Entity, With<ActiveSelection>>;
type RotateGizmoQuery<'w, 's> =
    Query<'w, 's, (Entity, &'w GizmoAxis, &'w ChildOf), With<RotateGizmo>>;

type RotateGizmoQueryWTransform<'w, 's> =
    Query<'w, 's, (Entity, &'w mut Transform, &'w GlobalTransform), With<RotateGizmoParent>>;
type TransformQuery<'w, 's> =
    Query<'w, 's, (&'w mut Transform, &'w GlobalTransform, Entity), Without<GizmoCamera>>;
type GizmoMeshNameQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'w GizmoMesh>,
        Option<&'w IconProxy>,
        &'w Name,
    ),
>;
type ParentQuery<'w, 's> = Query<'w, 's, &'w ChildOf>;
//
// ------------------------------------------------------------------------

pub fn handle_rotate_input(
    drag_state: ResMut<DragState>,
    selected_option: ResMut<NewGizmoType>,
    user_input: Res<UserInput>,
    selection_query: Query<Entity, With<ActiveSelection>>,
    mut init_drag_event: MessageWriter<RotateInitDragEvent>,
    mut dragging_event: MessageWriter<RotateDraggingEvent>,
    mut drag_ended_event: MessageWriter<RotateResetDragEvent>,
) {
    if !user_input.mouse_left.any {
        return;
    }

    if !matches!(**selected_option, GizmoType::Rotate) {
        // Gizmo value for Rotate
        return;
    }

    if selection_query.single().is_err() {
        return;
    }

    // Setup drag
    if user_input.mouse_left.just_pressed && !drag_state.dragging & !user_input.mouse_over_egui {
        init_drag_event.write(RotateInitDragEvent);
    }
    // Dragging
    else if user_input.mouse_left.pressed && drag_state.dragging {
        dragging_event.write(RotateDraggingEvent);
    }
    // Reset Drag
    else if user_input.mouse_left.just_released && drag_state.dragging {
        drag_ended_event.write(RotateResetDragEvent);
    }
}

pub fn handle_init_rotate_drag(
    mut events: MessageReader<RotateInitDragEvent>,
    mut drag_state: ResMut<DragState>,
    resources: (Res<CursorWindowPos>, Res<RaycastCursorPos>),
    mut duplicate_event_writer: MessageWriter<RequestDuplicateAllSelectionEvent>,
    user_input: Res<UserInput>,
    mut gizmo_visibility_query: Query<(&GizmoAxis, Mut<Visibility>)>,
    mut queries: ParamSet<(
        ActiveSelectionQuery,
        RotateGizmoQuery,
        ParentQuery,
        TransformQuery,
        GizmoMeshNameQuery,
        RotateGizmoQueryWTransform,
    )>,
    interactions: Query<
        (Entity, Option<&GizmoMesh>, &Name, &PickingInteraction),
        Changed<PickingInteraction>,
    >,
) {
    let (cursor_2d, raycast_cursor_pos) = resources;

    for _event in events.read() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Init rotate drag event",
        );

        // Step 1: Perform Raycast to find the hit entity
        let (entity, hit_type) = raycast_at_cursor(interactions);

        if hit_type == HitType::None || hit_type == HitType::Mesh || entity.is_none() {
            return;
        }

        // Step 2: Get the selected entity
        let selection_query = queries.p0();
        let Ok(_selection_entity) = selection_query.single() else {
            return;
        };

        let Some(raycast_target) = entity else {
            return;
        };

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Just pressed 'Left' and not dragging"
        );

        // Step 3: Get Gizmo Axis and Parent information
        if let Ok((_gizmo_entity, gizmo_axis, gizmo_parent)) = queries.p1().get(raycast_target) {
            let gizmo_axis = *gizmo_axis;

            let actual_parent = gizmo_parent.parent();

            hide_unselected_axes(gizmo_axis, &mut gizmo_visibility_query);

            let mut query_p3 = queries.p3();
            let Ok((parent_transform, parent_global_transform, _)) =
                query_p3.get_mut(actual_parent)
            else {
                return;
            };

            drag_state.initial_selection_rotation = parent_transform.rotation;
            drag_state.raycast_position = raycast_cursor_pos.position;
            drag_state.initial_cursor_position = cursor_2d.position;
            drag_state.gizmo_position = parent_global_transform.translation();
            drag_state.dragging = true;
            drag_state.locked_axis = Some(gizmo_axis);

            // Compute vector from gizmo to hit point
            let hit_vec = (raycast_cursor_pos.position - drag_state.gizmo_position).normalize();
            drag_state.prev_hit_dir = hit_vec;

            // Get and store initial gizmo rotation
            if let Ok((_, _gizmo_transform, gizmo_world_transform)) = queries.p5().single() {
                let (_, initial_gizmo_rotation, _) =
                    gizmo_world_transform.to_scale_rotation_translation();
                drag_state.initial_gizmo_rotation = initial_gizmo_rotation;
            } else {
                log!(
                    LogType::Editor,
                    LogLevel::Error,
                    LogCategory::Entity,
                    "Couldn't get gizmo transform"
                );
            }

            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Input,
                "Begin dragging at: {:?}",
                drag_state.locked_axis
            );

            // Step 7: Handle duplication if Shift key is pressed
            if user_input.shift_left.pressed {
                log!(
                    LogType::Editor,
                    LogLevel::Info,
                    LogCategory::Input,
                    "Duplicate entity"
                );
                duplicate_event_writer.write(RequestDuplicateAllSelectionEvent);
            }
        } else {
            return;
        }
    }
}

fn show_unselected_axes(gizmo_query: &mut Query<Mut<Visibility>>) {
    for mut visibility in gizmo_query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

// Function to hide unselected axes
fn hide_unselected_axes(
    selected_axis: GizmoAxis,
    gizmo_query: &mut Query<(&GizmoAxis, Mut<Visibility>)>,
) {
    for (axis, mut visibility) in gizmo_query.iter_mut() {
        *visibility = if *axis == selected_axis {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn handle_rotate_dragging(
    event: On<Pointer<Drag>>,
    targets: Query<&GizmoOf>,
    camera_query: Query<(&GlobalTransform, &Camera), With<GizmoCamera>>,
    mut objects: Query<&mut Transform, Without<GizmoCamera>>,
    global_transforms: Query<&GlobalTransform>,
    active_selection: Query<Entity, With<ActiveSelection>>,
    other_selected: Query<Entity, (With<Selected>, Without<ActiveSelection>)>,
    parents: Query<&ChildOf>,
    gizmo_snap: Res<GizmoSnap>,
    selected: Res<NewGizmoConfig>,
    gizmo_data: Query<(&GizmoAxis, Option<&GizmoConfig>)>,
    mut accrued: Local<Vec2>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok((gizmo_axis, config)) = gizmo_data.get(event.entity) else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "Gizmo Axis data not found for Gizmo entity {:?}",
            event.entity
        );
        return;
    };
    let GizmoConfig::Rotate {
        speed_scale,
        distance_scale: _,
        mode: _,
    } = config.cloned().unwrap_or(selected.rotation())
    else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "Gizmo Config for rotation was not a Rotation Config",
        );
        return;
    };

    let free_rotate_speed = 0.3 * speed_scale;

    *accrued += event.delta * free_rotate_speed;

    if accrued.x.abs() < gizmo_snap.rotate_value && accrued.y.abs() < gizmo_snap.rotate_value {
        return;
    }
    let accrued_x_degrees = accrued.x;
    let accrued_y_degrees = accrued.y;

    let x_step = snap_roation(accrued_x_degrees, gizmo_snap.rotate_value);
    let y_step = snap_roation(accrued_y_degrees, gizmo_snap.rotate_value);

    let delta_x = x_step.to_radians();
    let delta_y = y_step.to_radians();
    let Ok(_target) = targets.get(event.entity) else {
        log(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Debug,
            format!("Rotaion Gizmo({})'s Target not found", event.entity.index()),
        );
        return;
    };
    let Ok((camera_transform, camera)) = camera_query.single() else {
        log!(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Debug,
            "Gizmo Camera not found for rotation drag"
        );
        return;
    };
    let effective_delta_x = delta_x;
    let effective_delta_y = delta_y;

    let rotation_delta = Quat::from_axis_angle(camera_transform.up().as_vec3(), delta_x)
        * Quat::from_axis_angle(camera_transform.right().as_vec3(), delta_y);

    let Ok(click_ray) = camera.viewport_to_world(camera_transform, event.pointer_location.position)
    else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Failed to convert viewport to world coordinates for pointer location: {:?}",
            event.pointer_location.position
        };
        return;
    };
    let mut all_selected_entities = Vec::new();
    all_selected_entities.extend(active_selection.iter());
    all_selected_entities.extend(other_selected.iter());

    if all_selected_entities.is_empty() {
        return;
    }

    let mut root_entities = Vec::new();
    for &entity in &all_selected_entities {
        let mut is_child_of_selected = false;
        if let Ok(parent) = parents.get(entity) {
            if all_selected_entities.contains(&parent.parent()) {
                is_child_of_selected = true;
            }
        }
        if !is_child_of_selected {
            root_entities.push(entity);
        }
    }

    let origin = {
        if let Some(active_entity) = active_selection.iter().next() {
            if let Ok(active_global_transform) = global_transforms.get(active_entity) {
                active_global_transform.translation()
            } else {
                return;
            }
        } else {
            return;
        }
    };

    let final_rotation = match gizmo_axis {
        GizmoAxis::All => rotation_delta,
        GizmoAxis::X => {
            let mut delta = effective_delta_y;
            if origin.x > camera_transform.translation().x {
                delta = -delta;
            }

            Quat::from_rotation_x(delta)
        }
        GizmoAxis::Y => {
            let mut delta = effective_delta_x;
            if origin.y > camera_transform.translation().y {
                delta = -delta;
            }

            Quat::from_rotation_y(delta)
        }
        GizmoAxis::Z => {
            let (pitch, roll, yaw) = rotation_delta.to_euler(bevy::math::EulerRot::XZY);
            let mut delta = roll;
            if let Some(hit_distance) = click_ray
                .intersect_plane(Vec3::new(0., 0., origin.z), InfinitePlane3d::new(Vec3::Z))
            {
                let hit_point = camera_transform.translation() + click_ray.direction * hit_distance;
                let y_diff = origin.y - hit_point.y;
                let x_diff = origin.x - hit_point.x;
                delta += yaw * y_diff.signum();
                delta += pitch * x_diff.signum();
            }
            if origin.z > camera_transform.translation().z {
                delta = -delta;
            }
            Quat::from_rotation_z(delta)
        }
        GizmoAxis::None => {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Debug,
                "Rotation Gizmo Axis None Should not happen",
            );
            Quat::IDENTITY
        }
    };
    for &entity in &root_entities {
        if let Ok(mut entity_transform) = objects.get_mut(entity) {
            let relative_pos = entity_transform.translation - origin;
            let rotated_relative_pos = final_rotation * relative_pos;
            entity_transform.translation = origin + rotated_relative_pos;
            entity_transform.rotation = final_rotation * entity_transform.rotation;
        }
    }
    *accrued = Vec2::ZERO;
}

fn snap_roation(value: f32, inc: f32) -> f32 {
    if inc == 0.0 {
        value
    } else {
        (value / inc).round() * inc
    }
}

pub fn test_click_trigger(click: On<Pointer<Press>>, query: Query<&Name>) {
    let name = query.get(click.entity);
    println!(
        "Click on {:?} Triggered: {}\n, {:?}",
        name,
        click.entity.index(),
        click
    );
}

pub fn handle_rotate_reset(
    mut events: MessageReader<RotateResetDragEvent>,
    mut drag_state: ResMut<DragState>,
    selection_query: Query<Entity, With<ActiveSelection>>,
    transform_query: Query<(&mut Transform, &GlobalTransform, Entity), Without<GizmoCamera>>,
    mut gizmo_visibility_query: Query<Mut<Visibility>>,
) {
    for RotateResetDragEvent in events.read() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Rotation drag reset event",
        );
        let mut final_position = None;
        if let Some(selection_entity) = selection_query.iter().next() {
            if let Ok((_selection_transform, selection_global_transform, _)) =
                transform_query.get(selection_entity)
            {
                final_position = Some(selection_global_transform.translation());
            }
        }
        show_unselected_axes(&mut gizmo_visibility_query);

        drag_state.dragging = false;
        drag_state.locked_axis = None;
        drag_state.drag_ended = true;

        if let Some(position) = final_position {
            drag_state.raycast_position = position;
        }

        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Finish dragging"
        );
    }
}
