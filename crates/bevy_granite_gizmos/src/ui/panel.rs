use bevy::{
    ecs::{query::With, system::Query},
    prelude::{Res, ResMut},
};
use bevy_egui::{egui, EguiContexts};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

use crate::{
    gizmos::{
        vertex::{SelectedVertex, VertexMarker, VertexSelectionState, VertexVisualizationConfig},
        GizmoConfig, GizmoMode, GizmoSnap, GizmoType, Gizmos, NewGizmoConfig, NewGizmoType,
    },
    ActiveSelection,
};

pub fn editor_gizmos_ui(
    mut contexts: EguiContexts,
    mut selected_option: ResMut<NewGizmoType>,
    mut gizmo_snap: ResMut<GizmoSnap>,
    mut config: ResMut<NewGizmoConfig>,
    mut vertex_config: ResMut<VertexVisualizationConfig>,
    vertex_state: Res<VertexSelectionState>,
    selected_entity: Query<&Gizmos, With<ActiveSelection>>,
    mut gizmos: Query<&mut GizmoConfig>,
    selected_vertices: Query<
        (&VertexMarker, &bevy::prelude::GlobalTransform),
        With<SelectedVertex>,
    >,
) {
    let small_spacing = 1.;
    let spacing = 4.;
    let large_spacing = 6.;
    egui::Window::new("Gizmos")
        .resizable(false)
        .title_bar(false)
        .default_pos(egui::pos2(20.0, 90.0))
        .show(
            contexts.ctx_mut().expect("there to alway be a contex"),
            |ui| {
                let mut active = selected_option.as_mut().0;
                let mut mode = config.mode;
                let mut local = None;
                let mut changed = false;
                ui.vertical(|ui| {
                    if let Ok(selected) = selected_entity.single() {
                        for entity in selected.entities() {
                            if let Ok(local_config) = gizmos.get(*entity) {
                                active = local_config.gizmo_type();
                                mode = local_config.mode();
                                local = Some(*entity);
                            }
                        }
                    }
                    ui.set_max_width(150.);
                    changed |= ui
                        .radio_value(&mut active, GizmoType::Pointer, "Pointer")
                        .changed();

                    // Vertex visualization options for Pointer mode
                    if matches!(active, GizmoType::Pointer) {
                        ui.add_space(spacing);
                        if ui
                            .checkbox(&mut vertex_config.enabled, "Show Verts")
                            .changed()
                        {
                            log!(
                                LogType::Editor,
                                LogLevel::Info,
                                LogCategory::Entity,
                                "Vertex visualization {}",
                                if vertex_config.enabled {
                                    "enabled"
                                } else {
                                    "disabled"
                                }
                            );
                        }

                        // Show info about selected vertices
                        if !selected_vertices.is_empty() {
                            ui.add_space(large_spacing);
                            ui.group(|ui| {
                            ui.add_space(small_spacing);

                            if selected_vertices.iter().count() == 1 {
                                if let Some((_vertex_marker, global_transform)) =
                                    selected_vertices.iter().next()
                                {
                                    let (scale, rot_quat, pos) = global_transform
                                        .to_scale_rotation_translation();
                                    let rot = rot_quat
                                        .to_euler(bevy::prelude::EulerRot::YXZ);

                                ui.add_space(spacing);
                                ui.label("Vertex");
                                ui.add_space(spacing);
                                    
                                    egui::Grid::new("vertex_transform_grid")
                                        .num_columns(4)
                                        .spacing([2.0, 2.0])
                                        .striped(false)
                                        .show(ui, |ui| {
                                            let drag_width = 60.0;
                                            
                                            ui.label("Position: ");
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut pos.x.clone()).speed(0.1).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut pos.y.clone()).speed(0.1).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut pos.z.clone()).speed(0.1).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.end_row();
                                            
                                            let rot_degrees = (
                                                rot.1.to_degrees(), 
                                                rot.0.to_degrees(), 
                                                rot.2.to_degrees(), 
                                            );
                                            ui.label("Rotation: ");
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut rot_degrees.0.clone()).speed(1.0).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut rot_degrees.1.clone()).speed(1.0).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut rot_degrees.2.clone()).speed(1.0).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.end_row();
                                            
                                            ui.label("Scale: ");
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut scale.x.clone()).speed(0.01).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut scale.y.clone()).speed(0.01).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut scale.z.clone()).speed(0.01).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                            ui.end_row();
                                        });

                                    ui.add_space(spacing);
                                    
                                    if ui.button("Copy").clicked() {
                                        let affine = global_transform.affine();
                                        let matrix = affine.matrix3;
                                        let translation = affine.translation;
                                        ui.ctx().copy_text(format!(
                                            "[{}, {}, {}, 0.0]\n[{}, {}, {}, 0.0]\n[{}, {}, {}, 0.0]\n[{}, {}, {}, 1.0]",
                                            matrix.x_axis.x, matrix.x_axis.y, matrix.x_axis.z,
                                            matrix.y_axis.x, matrix.y_axis.y, matrix.y_axis.z,
                                            matrix.z_axis.x, matrix.z_axis.y, matrix.z_axis.z,
                                            translation.x, translation.y, translation.z,
                                        ));
                                    }
                                }
                            }

                            // Show midpoint for multiple vertices
                            if let Some(midpoint) = vertex_state.midpoint_world {
                                let mut avg_rotation = bevy::prelude::Quat::IDENTITY;
                                let mut avg_scale = bevy::prelude::Vec3::ZERO;
                                let count = selected_vertices.iter().count() as f32;
                                
                                if count > 0.0 {
                                    for (_marker, transform) in selected_vertices.iter() {
                                        let (scale, rotation, _pos) = transform.to_scale_rotation_translation();
                                        avg_scale += scale;
                                        avg_rotation = if avg_rotation == bevy::prelude::Quat::IDENTITY {
                                            rotation
                                        } else {
                                            avg_rotation.slerp(rotation, 1.0 / count)
                                        };
                                    }
                                    avg_scale /= count;
                                }
                                
                                let rot = avg_rotation.to_euler(bevy::prelude::EulerRot::YXZ);
                                let rot_degrees = (
                                    rot.1.to_degrees(), 
                                    rot.0.to_degrees(), 
                                    rot.2.to_degrees(), 
                                );

                                ui.add_space(spacing);
                                ui.label("Midpoint");
                                ui.add_space(spacing);
                                
                                egui::Grid::new("midpoint_transform_grid")
                                    .num_columns(4)
                                    .spacing([2.0, 2.0])
                                    .striped(false)
                                    .show(ui, |ui| {
                                        let drag_width = 60.0;
                                        
                                        ui.label("Position: ");
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut midpoint.x.clone()).speed(0.1).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut midpoint.y.clone()).speed(0.1).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut midpoint.z.clone()).speed(0.1).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.end_row();
                                        
                                        ui.label("Rotation: ");
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut rot_degrees.0.clone()).speed(1.0).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut rot_degrees.1.clone()).speed(1.0).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut rot_degrees.2.clone()).speed(1.0).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.end_row();
                                        
                                        ui.label("Scale: ");
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut avg_scale.x.clone()).speed(0.01).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut avg_scale.y.clone()).speed(0.01).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.add_sized([drag_width, 20.0], egui::DragValue::new(&mut avg_scale.z.clone()).speed(0.01).max_decimals(2).min_decimals(2).fixed_decimals(2));
                                        ui.end_row();
                                    });

                                ui.add_space(spacing);
                                
                                // Copy midpoint matrix button
                                if ui.button("Copy").clicked() {
                                    let affine = bevy::math::Affine3A::from_scale_rotation_translation(
                                        avg_scale,
                                        avg_rotation,
                                        midpoint,
                                    );
                                    let matrix = affine.matrix3;
                                    let translation = affine.translation;
                                    ui.ctx().copy_text(format!(
                                        "[{}, {}, {}, 0.0]\n[{}, {}, {}, 0.0]\n[{}, {}, {}, 0.0]\n[{}, {}, {}, 1.0]",
                                        matrix.x_axis.x, matrix.x_axis.y, matrix.x_axis.z,
                                        matrix.y_axis.x, matrix.y_axis.y, matrix.y_axis.z,
                                        matrix.z_axis.x, matrix.z_axis.y, matrix.z_axis.z,
                                        translation.x, translation.y, translation.z,
                                    ));
                                }
                            }
                            });
                        }
                    }

                    ui.separator();
                    changed |= ui
                        .radio_value(&mut active, GizmoType::Transform, "Move")
                        .changed();
                    changed |= ui
                        .radio_value(&mut active, GizmoType::Rotate, "Rotate")
                        .changed();

                    if matches!(active, GizmoType::Transform) {
                        ui.add_space(spacing);
                        ui.label("Snap:");
                        ui.add_space(small_spacing);
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut gizmo_snap.transform_value)
                                    .speed(1.)
                                    .range(0.0..=360.0),
                            )
                            .changed();
                        ui.add_space(spacing);
                        egui::ComboBox::new("GizmoMode", "")
                            .selected_text(match mode {
                                GizmoMode::Local => "Local",
                                GizmoMode::Global => "Global",
                            })
                            .show_ui(ui, |ui| {
                                changed |= ui
                                    .selectable_value(&mut mode, GizmoMode::Local, "Local")
                                    .changed();
                                changed |= ui
                                    .selectable_value(&mut mode, GizmoMode::Global, "Global")
                                    .changed();
                            });
                    }

                    if matches!(active, GizmoType::Rotate) {
                        ui.add_space(spacing);
                        ui.label("Snap°:");
                        ui.add_space(small_spacing);
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut gizmo_snap.rotate_value)
                                    .speed(1.)
                                    .range(0.0..=360.0),
                            )
                            .changed();

                        ui.add_space(spacing);
                        egui::ComboBox::new("GizmoMode", "")
                            .selected_text(match mode {
                                GizmoMode::Local => "Local",
                                GizmoMode::Global => "Global",
                            })
                            .show_ui(ui, |ui| {
                                changed |= ui
                                    .selectable_value(&mut mode, GizmoMode::Local, "Local")
                                    .changed();
                                changed |= ui
                                    .selectable_value(&mut mode, GizmoMode::Global, "Global")
                                    .changed();
                            });
                    }
                });
                if changed {
                    if let Some(entity) = local {
                        let Ok(mut gizmo) = gizmos.get_mut(entity) else {
                            log!(
                                LogType::Editor,
                                LogLevel::Error,
                                LogCategory::Entity,
                                "Failed to get gizmo to update config"
                            );
                            return;
                        };
                        gizmo.set_type(active, &config);
                        gizmo.set_mode(mode);
                        config.mode = mode;
                        **selected_option = active;
                    } else {
                        config.mode = mode;
                        **selected_option = active;
                    }
                }
            },
        );
}
