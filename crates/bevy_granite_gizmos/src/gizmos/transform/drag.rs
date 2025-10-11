use super::TransformGizmo;
use crate::{
    gizmos::{GizmoOf, GizmoSnap},
    input::GizmoAxis,
    selection::{ActiveSelection, RequestDuplicateAllSelectionEvent, Selected},
    GizmoCamera,
};
use bevy::{
    asset::Assets,
    ecs::{
        component::Component, entity::ContainsEntity, hierarchy::{ChildOf, Children}, message::MessageWriter, observer::On, system::Commands
    },
    gizmos::{retained::Gizmo, GizmoAsset},
    picking::events::{Drag, DragEnd, DragStart, Pointer, Press},
    prelude::{
        Entity, GlobalTransform, Query, Res, ResMut, Resource, Transform, Vec3, With, Without,
    },
};
use bevy_granite_core::UserInput;
use bevy_granite_logging::{
    config::{LogCategory, LogLevel, LogType},
    log,
};

#[derive(Resource, Default)]
pub struct TransformDuplicationState {
    pub just_duplicated: bool,
}

pub fn drag_transform_gizmo(
    event: On<Pointer<Drag>>,
    mut command: Commands,
    targets: Query<&GizmoOf>,
    camera_query: Query<(Entity, &GlobalTransform, &bevy::camera::Camera), With<GizmoCamera>>,
    mut objects: Query<&mut Transform>,
    global_transforms: Query<&GlobalTransform>,
    parents: Query<&ChildOf>,
    active_selection: Query<Entity, With<ActiveSelection>>,
    other_selected: Query<Entity, (With<Selected>, Without<ActiveSelection>)>,
    gizmo_snap: Res<GizmoSnap>,
    gizmo_data: Query<(&GizmoAxis, &TransformGizmo, &InitialDragOffset)>,
    user_input: Res<UserInput>,
    mut duplication_state: ResMut<TransformDuplicationState>,
) {
    if event.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }

    if duplication_state.just_duplicated {
        duplication_state.just_duplicated = false;
        return;
    }
    let Ok((axis, typ, drag_offset)) = gizmo_data.get(event.entity) else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "Gizmo Axis data not found for Gizmo entity {:?}",
            event.entity
        );
        return;
    };

    let Ok((c_entity, camera_transform, camera)) = camera_query.single() else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo camera not found",
        };
        return;
    };

    let Ok(GizmoOf(target)) = targets.get(event.entity) else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo target not found for entity {:?}",
            event.entity
        };
        return;
    };
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

    // Filter out entities that are children of other selected entities
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

    if root_entities.is_empty() {
        log! {
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "No root entities to transform"
        };
        return;
    }

    let mut current_world_pos = {
        let Ok(target_transform) = objects.get(*target) else {
            log! {
                LogType::Editor,
                LogLevel::Error,
                LogCategory::Input,
                "Gizmo target transform not found for entity {:?}",
                target
            };
            return;
        };

        if let Ok(global_transform) = global_transforms.get(*target) {
            global_transform.translation()
        } else {
            target_transform.translation
        }
    };

    let (active_axis, normal) = match typ {
        TransformGizmo::Axis => (axis.to_vec3(), camera_transform.forward().as_vec3()),
        TransformGizmo::Plane => (axis.plane_as_vec3(), axis.to_vec3()),
    };

    current_world_pos -= drag_offset.offset();

    let Some(click_distance) = click_ray.intersect_plane(
        current_world_pos,
        bevy::math::primitives::InfinitePlane3d::new(normal),
    ) else {
        return;
    };

    let hit = click_ray.get_point(click_distance);
    let raw_delta = hit - current_world_pos;
    let mut world_delta = snap_gizmo(active_axis * raw_delta, gizmo_snap.transform_value);

    // Apply the delta to all root selected entities
    if world_delta.length() > 0.0 {
        for &entity in &root_entities {
            if let Ok(parent) = parents.get(entity) {
                if let Ok(parent_global) = global_transforms.get(parent.parent()) {
                    let parent_rotation_inv =
                        parent_global.to_scale_rotation_translation().1.inverse();
                    let parent_local_delta = parent_rotation_inv * world_delta;
                    world_delta = parent_local_delta;
                }
            }
            command.entity(entity).insert(TransitionDelta(world_delta));
        }
    }

    if user_input.ctrl_left.any {
        if let Ok(mut camera_transform) = objects.get_mut(c_entity) {
            camera_transform.translation += world_delta;
        }
    }
}

pub fn calculate_drag_offset(
    event: On<Pointer<DragStart>>,
    mut command: Commands,
    object: Query<&GlobalTransform, With<Children>>,
    gizmo_data: Query<(Entity, &GizmoOf), With<GizmoAxis>>,
) {
    if event.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }
    let Ok((entity, parent)) = gizmo_data.get(event.entity) else {
        log!(
            LogType::Editor,
            LogLevel::Warning,
            LogCategory::Input,
            "Gizmo Axis data not found for Gizmo entity {:?}",
            event.entity
        );
        return;
    };
    let Ok(object_transform) = object.get(parent.entity()) else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo target not found for entity {:?}",
            event.entity
        };
        return;
    };

    // Fallback to the center of the target object if can't resolve position.
    let cursor_postion = event.hit.position.unwrap_or(object_transform.translation());
    command.entity(entity).insert(InitialDragOffset(
        object_transform.translation() - cursor_postion,
    ));
}

pub fn drag_end_cleanup(
    event: On<Pointer<DragEnd>>,
    mut command: Commands,
    gizmo_data: Query<Entity, With<InitialDragOffset>>,
) {
    if event.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }
    for gizmo_entity in gizmo_data {
        command.entity(gizmo_entity).remove::<InitialDragOffset>();
    }
}

pub fn apply_transformations(
    mut command: Commands,
    objects: Query<(Entity, &mut Transform, &TransitionDelta)>,
) {
    for (entity, mut transform, transition_delta) in objects {
        transform.translation += transition_delta.0;
        command.entity(entity).remove::<TransitionDelta>();
    }
}

pub fn dragstart_transform_gizmo(
    event: On<Pointer<DragStart>>,
    targets: Query<&GizmoOf>,
    gizmo_data: Query<(&GizmoAxis, &TransformGizmo)>,
    user_input: Res<UserInput>,
    mut dispatch: MessageWriter<RequestDuplicateAllSelectionEvent>,
    mut duplication_state: ResMut<TransformDuplicationState>,
) {
    if user_input.mouse_middle.any || !user_input.shift_left.pressed {
        return;
    }
    let Ok(_) = gizmo_data.get(event.entity) else {
        return;
    };
    let Ok(GizmoOf(_target)) = targets.get(event.entity) else {
        return;
    };
    log!("Attempting Drag Duplicate");
    dispatch.write(RequestDuplicateAllSelectionEvent);
    duplication_state.just_duplicated = true;
}

fn snap_gizmo(value: Vec3, inc: f32) -> Vec3 {
    if inc == 0.0 {
        value
    } else {
        (value / inc).round() * inc
    }
}

pub fn draw_axis_lines(
    event: On<Pointer<Press>>,
    gizmo_data: Query<(&GizmoAxis, &GizmoOf, &TransformGizmo), With<TransformGizmo>>,
    mut bevy_gizmo: ResMut<Assets<GizmoAsset>>,
    mut commands: Commands,
    origin: Query<&GlobalTransform>,
) {
    if event.button != bevy::picking::pointer::PointerButton::Primary {
        return;
    }

    let Ok((axis, root, transform)) = gizmo_data.get(event.entity) else {
        return;
    };
    if let GizmoAxis::All = axis {
        return;
    }
    let Ok(origin) = origin.get(root.get()) else {
        log! {
            LogType::Editor,
            LogLevel::Error,
            LogCategory::Input,
            "Gizmo origin transform not found for entity {:?}",
            root.0
        };
        return;
    };
    let mut asset = GizmoAsset::new();
    match transform {
        TransformGizmo::Axis => {
            render_line(&mut asset, axis, origin);
        }
        TransformGizmo::Plane => {
            let (a, b) = axis.plane();
            render_line(&mut asset, &a, origin);
            render_line(&mut asset, &b, origin);
        }
    }

    commands.spawn((
        *axis,
        GizmoOf(root.0),
        Gizmo {
            handle: bevy_gizmo.add(asset),
            ..Default::default()
        },
        AxisLine,
    ));
}

// Fix for too thick of lines
// Thanks to 0xD21F
fn render_line(asset: &mut GizmoAsset, axis: &GizmoAxis, origin: &GlobalTransform) {
    let step = 10.0;
    let max_distance = 1000.0;
    let mut current = -max_distance;
    while current < max_distance {
        asset.line(
            origin.translation() + axis.to_vec3() * current,
            origin.translation() + axis.to_vec3() * (current + step),
            axis.color(),
        );
        current += step;
    }
}

pub fn cleanup_axis_line(
    mut commands: Commands,
    query: Query<Entity, With<AxisLine>>,
    user_input: Res<UserInput>,
) {
    if user_input.mouse_left.just_released {
        for entity in query.iter() {
            commands.entity(entity).try_despawn();
        }
    }
}

#[derive(Component)]
pub struct AxisLine;

#[derive(Component)]
pub struct TransitionDelta(Vec3);

#[derive(Component)]
pub struct InitialDragOffset(Vec3);

impl InitialDragOffset {
    pub fn offset(&self) -> Vec3 {
        self.0
    }
}
