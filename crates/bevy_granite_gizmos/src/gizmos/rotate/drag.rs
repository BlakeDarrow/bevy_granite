// Apply the SAME world rotation delta to ROOT ENTITIES ONLY
// Children inherit rotation automatically through hierarchy
use crate::{
    gizmos::{
        GizmoConfig, GizmoMesh, GizmoMode, GizmoOf, GizmoRoot, GizmoSnap, GizmoType, NewGizmoConfig, NewGizmoType,
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
    ecs::{observer::On, query::Changed},
    picking::{
        events::{Drag, Pointer, Press},
        hover::PickingInteraction,
        pointer::PointerButton,
    },
    prelude::{
        ChildOf, Entity, GlobalTransform, MessageReader, MessageWriter, Mut, Name, ParamSet, Quat,
        Query, Res, ResMut, Transform, Vec3, Visibility, With, Without,
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
            drag_state.accumulated_angle = 0.0;
            drag_state.last_snapped = 0.0;

            drag_state.prev_hit_dir = match gizmo_axis {
                GizmoAxis::All => {
                    (raycast_cursor_pos.position - drag_state.gizmo_position).normalize()
                }
                GizmoAxis::X | GizmoAxis::Y | GizmoAxis::Z => {
                    (raycast_cursor_pos.position - drag_state.gizmo_position).normalize()
                }
                GizmoAxis::None => Vec3::ZERO,
            };

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

/// ANGULAR movement for locked axis. We dont want pixel delta for locked axis.
/// Free rotate can use mouse delta
pub fn handle_rotate_dragging(
    event: On<Pointer<Drag>>,
    targets: Query<&GizmoOf>,
    camera_query: Query<(&GlobalTransform, &Camera), With<GizmoCamera>>,
    mut objects: Query<&mut Transform, Without<GizmoCamera>>,
    global_transforms: Query<&GlobalTransform>,
    active_selection: Query<Entity, With<ActiveSelection>>,
    other_selected: Query<Entity, (With<Selected>, Without<ActiveSelection>)>,
    parents: Query<&ChildOf>,
    _gizmo_snap: Res<GizmoSnap>,
    selected: Res<NewGizmoConfig>,
    gizmo_data: Query<(&GizmoAxis, &GizmoRoot)>,
    gizmo_config_query: Query<&GizmoConfig>,
    mut drag_state: ResMut<DragState>,
) {
    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Debug,
        "handle_rotate_dragging called for entity: {:?}",
        event.entity
    );
    
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok((gizmo_axis, gizmo_root)) = gizmo_data.get(event.entity) else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "Gizmo Axis data not found for Gizmo entity {:?}",
            event.entity
        );
        return;
    };
    
    // Get config from parent gizmo entity
    let config = gizmo_config_query.get(gizmo_root.0).ok();
    
    log!(
        LogType::Editor,
        LogLevel::Info,
        LogCategory::Debug,
        "Gizmo config: {:?}, gizmo_axis: {:?}",
        config,
        gizmo_axis
    );
    
    let GizmoConfig::Rotate {
        speed_scale,
        distance_scale: _,
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

    let free_rotate_speed = 0.01 * speed_scale;
    let locked_rotate_speed = 1.0 * speed_scale;

    let Ok(target) = targets.get(event.entity) else {
        log(
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Debug,
            format!("Rotation Gizmo({})'s Target not found", event.entity.index()),
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
    
    // Get target rotation for local/global mode
    let target_rotation = if let Ok(global_transform) = global_transforms.get(target.0) {
        global_transform.to_scale_rotation_translation().1
    } else {
        if let Ok(transform) = objects.get(target.0) {
            transform.rotation
        } else {
            Quat::IDENTITY
        }
    };

    let (final_rotation, local_axis) = match gizmo_axis {
        GizmoAxis::All => {
            let delta_x = event.delta.x * free_rotate_speed;
            let delta_y = event.delta.y * free_rotate_speed;
            
            let snapped_delta_x = snap_roation(delta_x, _gizmo_snap.rotate_value.to_radians());
            let snapped_delta_y = snap_roation(delta_y, _gizmo_snap.rotate_value.to_radians());
            
            if snapped_delta_x.abs() < f32::EPSILON && snapped_delta_y.abs() < f32::EPSILON {
                return;
            }
            
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Debug,
                "Free rotation (All axis) - mode: {:?}",
                mode
            );
            
            let rotation = Quat::from_axis_angle(camera_transform.up().as_vec3(), snapped_delta_x)
                * Quat::from_axis_angle(camera_transform.right().as_vec3(), snapped_delta_y);
            (rotation, None)
        }
        GizmoAxis::X | GizmoAxis::Y | GizmoAxis::Z => {
            let axis = match gizmo_axis {
                GizmoAxis::X => Vec3::X,
                GizmoAxis::Y => Vec3::Y,
                GizmoAxis::Z => Vec3::Z,
                _ => return,
            };
            
            log!(
                LogType::Editor,
                LogLevel::Info,
                LogCategory::Debug,
                "Locked axis rotation: {:?}, mode: {:?}",
                gizmo_axis,
                mode
            );
            
            // Apply local/global mode transformation
            let world_axis = match mode {
                GizmoMode::Local => {
                    target_rotation * axis
                }
                GizmoMode::Global => {
                    axis
                }
            };

            let Ok(ray) = camera.viewport_to_world(camera_transform, event.pointer_location.position) else {
                log! {
                    LogType::Editor,
                    LogLevel::Error,
                    LogCategory::Input,
                    "Failed to convert viewport to world coordinates for pointer location: {:?}",
                    event.pointer_location.position
                };
                return;
            };

            let ray_origin = ray.origin;
            let ray_direction = ray.direction;
            let plane_normal = world_axis;
            
            let ray_dir_dot = ray_direction.dot(plane_normal);
            if ray_dir_dot.abs() < 1e-6 {
                return; // Ray parallel to plane
            }

            let t = (origin - ray_origin).dot(plane_normal) / ray_dir_dot;
            let hit_pos = ray_origin + ray_direction * t;
            let prev_vec = drag_state.prev_hit_dir;
            let curr_vec = (hit_pos - origin).normalize();
            
            if prev_vec.is_nan() || curr_vec.is_nan() || prev_vec.length_squared() < 1e-6 || curr_vec.length_squared() < 1e-6 {
                drag_state.prev_hit_dir = curr_vec;
                return;
            }
            
            let dot_product = prev_vec.dot(curr_vec);
            if dot_product < 0.95 {
                drag_state.prev_hit_dir = curr_vec;
                return;
            }
            
            let unsigned_angle = prev_vec.angle_between(curr_vec);
            if unsigned_angle.is_nan() || !unsigned_angle.is_finite() {
                return;
            }
            
            let angle_threshold = 0.001; // ~0.057 degrees
            if unsigned_angle.abs() < angle_threshold {
                return; 
            }
            
            let direction = prev_vec.cross(curr_vec).dot(world_axis).signum();
            let signed_angle = unsigned_angle * direction * locked_rotate_speed;
            let rotation_delta = Quat::from_axis_angle(world_axis, signed_angle);
            
            drag_state.prev_hit_dir = curr_vec;
            
            (rotation_delta, Some((axis, signed_angle)))
        }
        GizmoAxis::None => {
            log!(
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Debug,
                "Rotation Gizmo Axis None Should not happen",
            );
            (Quat::IDENTITY, None)
        }
    };

    for &entity in &root_entities {
        if let Ok(mut entity_transform) = objects.get_mut(entity) {
            match mode {
                GizmoMode::Local => {
                    // In local mode, rotation is applied in local space (position doesn't change)
                    if let Some((local_axis, signed_angle)) = local_axis {
                        log!(
                            LogType::Editor,
                            LogLevel::Info,
                            LogCategory::Debug,
                            "Local mode: Rotating around {:?} by {} radians",
                            local_axis,
                            signed_angle
                        );
                        // Apply rotation in local space around the local axis
                        let local_rotation = Quat::from_axis_angle(local_axis, signed_angle);
                        entity_transform.rotation = entity_transform.rotation * local_rotation;
                    } else {
                        // Free rotation (GizmoAxis::All) - apply in world space
                        entity_transform.rotation = final_rotation * entity_transform.rotation;
                    }
                }
                GizmoMode::Global => {
                    // In global mode, rotation affects both position and rotation
                    let relative_pos = entity_transform.translation - origin;
                    let rotated_relative_pos = final_rotation * relative_pos;
                    entity_transform.translation = origin + rotated_relative_pos;
                    entity_transform.rotation = final_rotation * entity_transform.rotation;
                }
            }
        }
    }
}

#[allow(dead_code)]
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
