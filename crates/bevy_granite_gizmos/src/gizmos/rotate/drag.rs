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
        ray::{raycast_at_cursor, HitType, RaycastCursorLast, RaycastCursorPos},
        ActiveSelection, RequestDuplicateAllSelectionEvent, Selected,
    },
    GizmoCamera,
};
use bevy::{
    ecs::{observer::Trigger, query::Changed, system::Local},
    math::primitives::InfinitePlane3d,
    picking::{
        events::{Drag, Pointer, Pressed},
        hover::PickingInteraction,
        pointer::PointerButton,
    },
    prelude::{
        ChildOf, Children, Entity, EventReader, EventWriter, GlobalTransform, Mut, Name, ParamSet,
        Quat, Query, Res, ResMut, Transform, Vec2, Vec3, Visibility, With, Without,
    },
    render::camera::Camera,
};
use bevy_granite_core::{CursorWindowPos, IconProxy, UserInput};
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

// ------------------------------------------------------------------------
//
type CameraQuery<'w, 's> = Query<'w, 's, &'w Transform, With<GizmoCamera>>;
type ActiveSelectionQuery<'w, 's> = Query<'w, 's, Entity, With<ActiveSelection>>;
type RotateGizmoQuery<'w, 's> =
    Query<'w, 's, (Entity, &'w GizmoAxis, &'w ChildOf), With<RotateGizmo>>;

type RotateGizmoQueryWTransform<'w, 's> =
    Query<'w, 's, (Entity, &'w mut Transform, &'w GlobalTransform), With<RotateGizmoParent>>;
type NonActiveSelectionQuery<'w, 's> =
    Query<'w, 's, Entity, (With<Selected>, Without<ActiveSelection>)>;
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
type ChildrenQuery<'w, 's> = Query<'w, 's, &'w Children>;
//
// ------------------------------------------------------------------------

pub fn handle_rotate_input(
    drag_state: ResMut<DragState>,
    selected_option: ResMut<NewGizmoType>,
    user_input: Res<UserInput>,
    selection_query: Query<Entity, With<ActiveSelection>>,
    mut init_drag_event: EventWriter<RotateInitDragEvent>,
    mut dragging_event: EventWriter<RotateDraggingEvent>,
    mut drag_ended_event: EventWriter<RotateResetDragEvent>,
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
    mut events: EventReader<RotateInitDragEvent>,
    mut drag_state: ResMut<DragState>,
    resources: (
        Res<CursorWindowPos>,
        ResMut<RaycastCursorLast>,
        ResMut<RaycastCursorPos>,
    ),
    mut duplicate_event_writer: EventWriter<RequestDuplicateAllSelectionEvent>,
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
        (
            Entity,
            Option<&GizmoMesh>,
            &Name,
            &PickingInteraction,
        ),
        Changed<PickingInteraction>,
    >,
) {
    let (cursor_2d, mut raycast_cursor_last_pos, mut raycast_cursor_pos) = resources;

    for _event in events.read() {
        log!(
            LogType::Editor,
            LogLevel::Info,
            LogCategory::Input,
            "Init rotate drag event",
        );

        // Step 1: Perform Raycast to find the hit entity
        let (entity, hit_type) = raycast_at_cursor(interactions);

        if hit_type == HitType::None
            || hit_type == HitType::Mesh
            || entity.is_none()
        {
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
    event: Trigger<Pointer<Drag>>,
    targets: Query<&GizmoOf>,
    camera_query: Query<(&GlobalTransform, &Camera), With<GizmoCamera>>,
    mut objects: Query<&mut Transform, Without<GizmoCamera>>,
    gizmo_snap: Res<GizmoSnap>,
    selected: Res<NewGizmoConfig>,
    gizmo_data: Query<(&GizmoAxis, Option<&GizmoConfig>)>,
    mut accrued: Local<Vec2>,
) {
    // return if not dragging with primary button
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok((gizmo_axis, config)) = gizmo_data.get(event.target) else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "Gizmo Axis data not found for Gizmo entity {:?}",
            event.target
        );
        return;
    };
    let GizmoConfig::Rotate {
        speed_scale,
        distance_scale,
        mode,
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
    let x_step = snap_roation(accrued.x, gizmo_snap.rotate_value);
    let y_step = snap_roation(accrued.y, gizmo_snap.rotate_value);
    let delta_x = x_step.to_radians();
    let delta_y = y_step.to_radians();
    let Ok(target) = targets.get(event.target) else {
        log(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Debug,
            format!("Rotaion Gizmo({})'s Target not found", event.target.index()),
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
    let (pitch, roll, yaw) = rotation_delta.to_euler(bevy::math::EulerRot::XZY);
    let Ok(mut transform) = objects.get_mut(**target) else {
        log!(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Debug,
            "Target entity {:?} missing Transform for rotation drag",
            **target
        );
        return;
    };
    match gizmo_axis {
        GizmoAxis::All => {
            transform.rotate(rotation_delta);
        }
        GizmoAxis::X => {
            let mut delta = pitch;
            if let Some(hit_distance) = click_ray.intersect_plane(
                Vec3::new(transform.translation.x, 0., 0.),
                InfinitePlane3d::new(Vec3::X),
            ) {
                let hit_point = camera_transform.translation() + click_ray.direction * hit_distance;
                let z_diff = transform.translation.z - hit_point.z;
                let y_diff = transform.translation.y - hit_point.y;
                delta += roll * z_diff.signum();
                delta += yaw * y_diff.signum();
            }
            if transform.translation.x > camera_transform.translation().x {
                delta = -delta;
            }
            transform.rotate_x(delta);
        }
        GizmoAxis::Y => {
            let mut delta = yaw;
            if let Some(hit_distance) = click_ray.intersect_plane(
                Vec3::new(0., transform.translation.y, 0.),
                InfinitePlane3d::new(Vec3::Y),
            ) {
                let hit_point = camera_transform.translation() + click_ray.direction * hit_distance;
                let z_diff = transform.translation.z - hit_point.z;
                let x_diff = transform.translation.x - hit_point.x;
                delta += pitch * x_diff.signum();
                delta += roll * z_diff.signum();
            }
            if transform.translation.y > camera_transform.translation().y {
                delta = -delta;
            }
            transform.rotate_y(delta);
        }
        GizmoAxis::Z => {
            let mut delta = roll;
            if let Some(hit_distance) = click_ray.intersect_plane(
                Vec3::new(0., 0., transform.translation.z),
                InfinitePlane3d::new(Vec3::Z),
            ) {
                let hit_point = camera_transform.translation() + click_ray.direction * hit_distance;
                let y_diff = transform.translation.y - hit_point.y;
                let x_diff = transform.translation.x - hit_point.x;
                delta += yaw * y_diff.signum();
                delta += pitch * x_diff.signum();
            }
            if transform.translation.z > camera_transform.translation().z {
                delta = -delta;
            }
            transform.rotate_z(delta);
        }
        GizmoAxis::None => {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Debug,
                "Rotation Gizmo Axis None Should not happen",
            )
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

pub fn test_click_trigger(click: Trigger<Pointer<Pressed>>, query: Query<&Name>) {
    let name = query.get(click.target);
    println!(
        "Click on {:?} Triggered: {}\n, {:?}",
        name,
        click.target.index(),
        click
    );
}

fn apply_independent_rotation(
    queries: &mut ParamSet<(
        CameraQuery,
        ActiveSelectionQuery,
        NonActiveSelectionQuery,
        TransformQuery,
        RotateGizmoQueryWTransform,
        ChildrenQuery,
        ParentQuery,
    )>,
    all_selected_entities: &[Entity],
    rotation_delta: Quat,
) {
    // Phase 1a: Get original global transforms
    let mut original_data = std::collections::HashMap::new();
    {
        let transform_query = queries.p3();
        for &entity in all_selected_entities {
            if let Ok((_, global_transform, _)) = transform_query.get(entity) {
                let (scale, rotation, translation) =
                    global_transform.to_scale_rotation_translation();
                original_data.insert(entity, (scale, rotation, translation));
            }
        }
    }

    // Phase 1b: Get parent relationships
    let mut parent_map = std::collections::HashMap::new();
    {
        let parent_query = queries.p6();
        for &entity in all_selected_entities {
            if let Ok(parent) = parent_query.get(entity) {
                parent_map.insert(entity, parent.parent());
            }
        }
    }

    // Phase 2: Calculate what each entity's final local transform should be
    let mut final_local_transforms = std::collections::HashMap::new();

    for &entity in all_selected_entities {
        if let Some((scale, rotation, translation)) = original_data.get(&entity) {
            // Target: same global position, rotated rotation
            let target_global_rotation = rotation_delta * *rotation;
            let target_global_position = *translation; // STAY PUT!

            if let Some(parent_entity) = parent_map.get(&entity) {
                // Child entity - need parent's current state
                let (parent_rotation, parent_translation) =
                    if all_selected_entities.contains(parent_entity) {
                        // Parent is selected, use its rotated state
                        if let Some((_, parent_orig_rotation, parent_orig_translation)) =
                            original_data.get(parent_entity)
                        {
                            (
                                rotation_delta * *parent_orig_rotation,
                                *parent_orig_translation,
                            )
                        } else {
                            continue; // Skip if can't get parent data
                        }
                    } else {
                        // Parent is NOT selected, get its current transform
                        // We need to get this from a fresh query since it's not in original_data
                        continue; // We'll handle this in a separate phase
                    };

                // Convert child's target global state to local relative to parent's state
                let local_position =
                    parent_rotation.inverse() * (target_global_position - parent_translation);
                let local_rotation = parent_rotation.inverse() * target_global_rotation;

                final_local_transforms.insert(entity, (local_position, local_rotation, *scale));
            } else {
                // Root entity - local = global
                final_local_transforms
                    .insert(entity, (*translation, target_global_rotation, *scale));
            }
        }
    }

    // Phase 2b: Handle children whose parents are NOT selected
    {
        let transform_query = queries.p3();
        for &entity in all_selected_entities {
            if final_local_transforms.contains_key(&entity) {
                continue; // Already handled
            }

            if let Some((scale, rotation, translation)) = original_data.get(&entity) {
                let target_global_rotation = rotation_delta * *rotation;
                let target_global_position = *translation;

                if let Some(parent_entity) = parent_map.get(&entity) {
                    // Get parent's current transform
                    if let Ok((_, parent_global, _)) = transform_query.get(*parent_entity) {
                        let (_, parent_rotation, parent_translation) =
                            parent_global.to_scale_rotation_translation();

                        let local_position = parent_rotation.inverse()
                            * (target_global_position - parent_translation);
                        let local_rotation = parent_rotation.inverse() * target_global_rotation;

                        final_local_transforms
                            .insert(entity, (local_position, local_rotation, *scale));
                    }
                }
            }
        }
    }

    // Phase 3: Apply all transforms simultaneously
    {
        let mut transform_query = queries.p3();
        for &entity in all_selected_entities {
            if let Some((pos, rot, scale)) = final_local_transforms.get(&entity) {
                if let Ok((mut transform, _, _)) = transform_query.get_mut(entity) {
                    transform.translation = *pos;
                    transform.rotation = *rot;
                    transform.scale = *scale;
                }
            }
        }
    }
}

pub fn handle_rotate_reset(
    mut events: EventReader<RotateResetDragEvent>,
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
