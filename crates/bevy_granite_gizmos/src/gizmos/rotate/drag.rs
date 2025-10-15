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
    mut gizmo_visibility_query: Query<(&GizmoAxis, &mut Visibility, &GizmoRoot), With<RotateGizmo>>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok((gizmo_axis, gizmo_root)) = gizmo_data.get(event.entity) else {
        return;
    };
    
    if !drag_state.dragging {
        drag_state.dragging = true;
        for (axis, mut visibility, root) in gizmo_visibility_query.iter_mut() {
            if root.0 == gizmo_root.0 {
                *visibility = if *axis == *gizmo_axis {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
    
    let config = gizmo_config_query.get(gizmo_root.0).ok();
    
    let GizmoConfig::Rotate {
        speed_scale,
        distance_scale: _,
        mode,
    } = config.cloned().unwrap_or(selected.rotation())
    else {
        return;
    };

    let free_rotate_speed = 0.01 * speed_scale;
    let locked_rotate_speed = 1.0 * speed_scale;

    let Ok(target) = targets.get(event.entity) else {
        return;
    };
    let Ok((camera_transform, camera)) = camera_query.single() else {
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
            let snap_increment = _gizmo_snap.rotate_value.to_radians();
            
            if snap_increment > 0.0 {
                let delta_x = event.delta.x * free_rotate_speed;
                let delta_y = event.delta.y * free_rotate_speed;
                
                let rotation_magnitude = (delta_x * delta_x + delta_y * delta_y).sqrt();
                
                drag_state.accumulated_angle += rotation_magnitude;
                
                let delta_from_last_snap = drag_state.accumulated_angle - drag_state.last_snapped;
                if delta_from_last_snap.abs() >= snap_increment {
                    let snap_count = (delta_from_last_snap / snap_increment).trunc();
                    let snapped_magnitude = snap_count * snap_increment;
                    
                    let normalized_delta_x = if rotation_magnitude > f32::EPSILON {
                        delta_x / rotation_magnitude
                    } else {
                        0.0
                    };
                    let normalized_delta_y = if rotation_magnitude > f32::EPSILON {
                        delta_y / rotation_magnitude
                    } else {
                        0.0
                    };
                    
                    let snapped_delta_x = normalized_delta_x * snapped_magnitude;
                    let snapped_delta_y = normalized_delta_y * snapped_magnitude;
                    
                    drag_state.last_snapped += snapped_magnitude;
                    let rotation = Quat::from_axis_angle(camera_transform.up().as_vec3(), snapped_delta_x)
                        * Quat::from_axis_angle(camera_transform.right().as_vec3(), snapped_delta_y);
                    (rotation, None)
                } else {
                    return;
                }
            } else {
                let delta_x = event.delta.x * free_rotate_speed;
                let delta_y = event.delta.y * free_rotate_speed;
                let rotation = Quat::from_axis_angle(camera_transform.up().as_vec3(), delta_x)
                    * Quat::from_axis_angle(camera_transform.right().as_vec3(), delta_y);
                (rotation, None)
            }
        }
        GizmoAxis::X | GizmoAxis::Y | GizmoAxis::Z => {
            let axis = match gizmo_axis {
                GizmoAxis::X => Vec3::X,
                GizmoAxis::Y => Vec3::Y,
                GizmoAxis::Z => Vec3::Z,
                _ => return,
            };
            
            let world_axis = match mode {
                GizmoMode::Local => {
                    target_rotation * axis
                }
                GizmoMode::Global => {
                    axis
                }
            };

            let Ok(ray) = camera.viewport_to_world(camera_transform, event.pointer_location.position) else {
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
            
            let snap_increment = _gizmo_snap.rotate_value.to_radians();
            let (snapped_angle, new_accumulated, new_last_snapped) = calculate_snap_rotation(
                signed_angle,
                drag_state.accumulated_angle,
                drag_state.last_snapped,
                snap_increment,
            );
            
            drag_state.accumulated_angle = new_accumulated;
            drag_state.last_snapped = new_last_snapped;
            drag_state.prev_hit_dir = curr_vec;
            
            if snapped_angle.abs() < f32::EPSILON {
                return;
            }
            
            let rotation_delta = Quat::from_axis_angle(world_axis, snapped_angle);
            
            (rotation_delta, Some((axis, snapped_angle)))
        }
        GizmoAxis::None => {
            (Quat::IDENTITY, None)
        }
    };

    for &entity in &root_entities {
        if let Ok(mut entity_transform) = objects.get_mut(entity) {
            match mode {
                GizmoMode::Local => {
                    if let Some((local_axis, signed_angle)) = local_axis {
                        let local_rotation = Quat::from_axis_angle(local_axis, signed_angle);
                        entity_transform.rotation = entity_transform.rotation * local_rotation;
                    } else {
                        entity_transform.rotation = final_rotation * entity_transform.rotation;
                    }
                }
                GizmoMode::Global => {
                    // Get the current global rotation
                    // Apply the rotation in global space
                    // Convert back to local space (accounting for parent rotation)
                    
                    let current_global_rotation = if let Ok(global_transform) = global_transforms.get(entity) {
                        global_transform.to_scale_rotation_translation().1
                    } else {
                        entity_transform.rotation
                    };
                    
                    let new_global_rotation = final_rotation * current_global_rotation;
                    
                    if let Ok(parent) = parents.get(entity) {
                        if let Ok(parent_global) = global_transforms.get(parent.parent()) {
                            let parent_rotation = parent_global.to_scale_rotation_translation().1;
                            entity_transform.rotation = parent_rotation.inverse() * new_global_rotation;
                        } else {
                            entity_transform.rotation = new_global_rotation;
                        }
                    } else {
                        entity_transform.rotation = new_global_rotation;
                    }
                }
            }
        }
    }
}

/// Calculates the snapped rotation angle based on accumulated rotation
fn calculate_snap_rotation(
    raw_delta: f32,
    accumulated: f32,
    last_snapped: f32,
    snap_increment: f32,
) -> (f32, f32, f32) {
    if snap_increment == 0.0 || snap_increment.abs() < f32::EPSILON {
        return (raw_delta, 0.0, 0.0);
    }

    let new_accumulated = accumulated + raw_delta;
    let delta_from_last_snap = new_accumulated - last_snapped;
    let snap_count = (delta_from_last_snap / snap_increment).trunc();
    
    if snap_count.abs() >= 1.0 {
        let snapped_angle = snap_count * snap_increment;
        let new_last_snapped = last_snapped + snapped_angle;
        (snapped_angle, new_accumulated, new_last_snapped)
    } else {
        (0.0, new_accumulated, last_snapped)
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
