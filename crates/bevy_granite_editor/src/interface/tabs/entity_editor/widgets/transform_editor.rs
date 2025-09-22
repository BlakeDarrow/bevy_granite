use crate::interface::tabs::EntityEditorTabData;
use bevy::prelude::{EulerRot, Quat, Vec3};
use bevy_egui::egui;
use bevy_granite_core::TransformData;
use bevy_granite_gizmos::GizmoAxis;
use std::f32::consts::PI;

// global_transform_data is serialized
#[derive(Default, PartialEq, Clone)]
pub struct EntityGlobalTransformData {
    pub global_transform_data: TransformData,
    pub transform_data_changed: bool,
    pub gizmo_axis: Option<GizmoAxis>,
    pub editing_rotation: [bool; 3],
    pub euler_degrees: Vec3,
    pub euler_radians: Vec3,
    pub last_synced_quat: Quat,
    // Not sure all this is needed for euler stability
}

impl EntityGlobalTransformData {
    pub fn clear(&mut self) {
        //log!(
        //    LogType::Editor,
        //    LogLevel::Info,
        //    LogCategory::UI,
        //    "EntityGlobalTransformData cleared"
        //);
        self.global_transform_data = TransformData::default();
        self.transform_data_changed = false;
        self.gizmo_axis = None;
        self.editing_rotation = [false; 3];
        self.euler_degrees = Vec3::ZERO;
        self.euler_radians = Vec3::ZERO;
        self.last_synced_quat = Quat::IDENTITY;
    }
}

pub fn entity_transform_widget(ui: &mut egui::Ui, data: &mut EntityEditorTabData) {
    let large_spacing = crate::UI_CONFIG.large_spacing;
    // --------------------------------------------------------------------
    // TRANSFORM
    // --------------------------------------------------------------------
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.add_space(large_spacing);
        ui.horizontal(|ui| {
            ui.add_space(large_spacing);
            display_transform_data(ui, data);
            ui.add_space(large_spacing);
        });
        ui.add_space(large_spacing);
    });
}

// FIX:
// button stuff is JANK for drag_spacing
fn display_transform_data(ui: &mut egui::Ui, data: &mut EntityEditorTabData) {
    let transform = &mut data.global_transform_data;
    let pos = &mut transform.global_transform_data.position;
    let scale = &mut transform.global_transform_data.scale;
    let quat_rot = &mut transform.global_transform_data.rotation;
    let changed = &mut transform.transform_data_changed;
    let editing = &mut transform.editing_rotation;
    let euler = &mut transform.euler_degrees;
    let euler_radians = &mut transform.euler_radians;
    let last_synced_quat = &mut transform.last_synced_quat;
    let gizmo_locked_axis = transform.gizmo_axis;
    let large_spacing = crate::UI_CONFIG.large_spacing;
    let small_spacing = crate::UI_CONFIG.small_spacing;
    let style = ui.ctx().style().clone();
    let default_font_id = egui::FontId::default();

    let font_id = style
        .text_styles
        .get(&egui::TextStyle::Button)
        .unwrap_or(&default_font_id);

    let btn_height = font_id.size + style.spacing.button_padding.y * 2.0;
    let drag_size = [60., btn_height];

    egui::Grid::new("transform_grid")
        .num_columns(3)
        .spacing([large_spacing, small_spacing])
        .striped(true)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // Position
            ui.vertical(|ui| {
                display_position_ui(ui, pos, changed, drag_size);
            });
            ui.end_row();

            // Rotation
            ui.vertical(|ui| {
                display_rotation_ui(
                    ui,
                    euler,
                    euler_radians,
                    quat_rot,
                    last_synced_quat,
                    changed,
                    editing,
                    gizmo_locked_axis,
                    drag_size,
                );
            });
            ui.end_row();

            // Scale
            ui.vertical(|ui| {
                display_scale_ui(ui, scale, changed, drag_size);
            });
            ui.end_row();
        });

    if !ui.input(|i| i.pointer.any_down()) {
        *editing = [false; 3];
    }
}

// TODO:
// take some of this and use it in the rotate gizmo?
//
// This also updates when user rotates via the rotate gizmo
// so we need to stabile the euler when user is dragging
// thats why it has both UI drag and world gizmo drag
fn display_rotation_ui(
    ui: &mut egui::Ui,
    euler: &mut Vec3,
    euler_radians: &mut Vec3,
    quat_rot: &mut Quat,
    last_synced_quat: &mut Quat,
    changed: &mut bool,
    editing: &mut [bool; 3],
    gizmo_locked_axis: Option<GizmoAxis>,
    drag_size: [f32; 2],
) {
    let spacing = crate::UI_CONFIG.large_spacing;
    ui.horizontal(|ui| {
        let label_width = (ui.available_width() / 5.) + spacing;
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(label_width, drag_size[1]), egui::Sense::hover());
        ui.painter().text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            "Rotation:",
            egui::FontId::default(),
            ui.visuals().text_color(),
        );

        egui::Grid::new("rotation_grid")
            .num_columns(4)
            .spacing([1.0, 0.0])
            .striped(true)
            .show(ui, |ui| {
                let mut euler_vals = normalize_euler_visual(*euler);
                let mut ui_changed = [false; 3];
                // FIX:
                // stabilize euler vals. Prefer only one channel to be pos/neg over flipping
                // the opposite 2
                // Draw UI for all 3 axes — always editable by user
                for i in 0..3 {
                    let drag_value = egui::DragValue::new(&mut euler_vals[i])
                        .speed(1.0)
                        .fixed_decimals(2);
                    let response = ui.add_sized(drag_size, drag_value);

                    // Add context menu for individual axis reset
                    response.context_menu(|ui| {
                        if ui.button("Reset").clicked() {
                            euler_vals[i] = 0.0;
                            ui_changed[i] = true;
                            ui.close();
                        }
                    });

                    editing[i] = response.dragged() || response.drag_started();
                    ui_changed[i] = ui_changed[i] || response.changed();
                }

                let zero = ui.add_sized(drag_size, egui::Button::new("Reset"));

                let is_editing = editing.iter().any(|&e| e);

                // Sync from quaternion to euler when not editing
                if !is_editing && *quat_rot != *last_synced_quat {
                    // Convert quaternion back to world-space Euler angles
                    // We need to decompose the world-space rotation back to individual axis rotations
                    let (x, y, z) = quat_rot.to_euler(EulerRot::YXZ); // Use YXZ to match our combination order
                    let degrees = [x, y, z].map(|r| r * 180.0 / PI);

                    let should_update_axis = |i| match gizmo_locked_axis {
                        Some(GizmoAxis::X) => i == 0,
                        Some(GizmoAxis::Y) => i == 1,
                        Some(GizmoAxis::Z) => i == 2,
                        Some(GizmoAxis::All) | Some(GizmoAxis::None) | None => true,
                    };

                    // Map back to X, Y, Z order for UI display
                    let euler_xyz = [degrees[1], degrees[0], degrees[2]]; // YXZ -> XYZ

                    for i in 0..3 {
                        if should_update_axis(i) {
                            euler[i] = closest_angle(euler[i], euler_xyz[i]);
                            euler_radians[i] = euler[i] * PI / 180.0;
                        }
                    }

                    *last_synced_quat = *quat_rot;
                }

                // Reset editing flags once user is done
                if !ui.input(|i| i.pointer.any_down()) {
                    *editing = [false; 3];
                }

                // Apply UI changes to euler + quat
                let mut dirty = false;
                for i in 0..3 {
                    if ui_changed[i] {
                        euler[i] = clamp_angle_360(euler_vals[i]);
                        euler_radians[i] = euler[i] * PI / 180.0;
                        dirty = true;
                    }
                }

                // Only apply quat update if not being manipulated externally
                if dirty
                    && (gizmo_locked_axis.is_none() || gizmo_locked_axis == Some(GizmoAxis::None))
                {
                    // Apply rotations in WORLD SPACE (global), not local space
                    // Build rotation by combining world-axis rotations
                    let x_rot = Quat::from_rotation_x(euler_radians.x);
                    let y_rot = Quat::from_rotation_y(euler_radians.y);
                    let z_rot = Quat::from_rotation_z(euler_radians.z);

                    // Combine rotations in world space order (Y * X * Z is common for world-space)
                    *quat_rot = y_rot * x_rot * z_rot;
                    *changed = true;
                    *last_synced_quat = *quat_rot;
                }

                if zero.clicked() {
                    *quat_rot = Quat::IDENTITY;
                    *euler_radians = Vec3::ZERO;
                    *last_synced_quat = Quat::IDENTITY;
                    *changed = true;
                    *euler = Vec3::ZERO;
                }
            });
    });
}

fn display_position_ui(ui: &mut egui::Ui, pos: &mut Vec3, changed: &mut bool, drag_size: [f32; 2]) {
    let spacing = crate::UI_CONFIG.large_spacing;
    ui.horizontal(|ui| {
        let label_width = (ui.available_width() / 5.) + spacing;
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(label_width, drag_size[1]), egui::Sense::hover());
        ui.painter().text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            "Position:",
            egui::FontId::default(),
            ui.visuals().text_color(),
        );

        egui::Grid::new("pos_grid")
            .num_columns(4)
            .spacing([1.0, 0.0])
            .striped(true)
            .show(ui, |ui| {
                let mut pos_x = pos.x;
                let mut pos_y = pos.y;
                let mut pos_z = pos.z;

                let x = ui.add_sized(
                    drag_size,
                    egui::DragValue::new(&mut pos_x)
                        .speed(0.1)
                        .fixed_decimals(2),
                );
                x.context_menu(|ui| {
                    if ui.button("Reset").clicked() {
                        pos_x = 0.0;
                        pos.x = pos_x;
                        *changed = true;
                        ui.close();
                    }
                });

                let y = ui.add_sized(
                    drag_size,
                    egui::DragValue::new(&mut pos_y)
                        .speed(0.1)
                        .fixed_decimals(2),
                );
                y.context_menu(|ui| {
                    if ui.button("Reset").clicked() {
                        pos_y = 0.0;
                        pos.y = pos_y;
                        *changed = true;
                        ui.close();
                    }
                });

                let z = ui.add_sized(
                    drag_size,
                    egui::DragValue::new(&mut pos_z)
                        .speed(0.1)
                        .fixed_decimals(2),
                );
                z.context_menu(|ui| {
                    if ui.button("Reset").clicked() {
                        pos_z = 0.0;
                        pos.z = pos_z;
                        *changed = true;
                        ui.close();
                    }
                });

                let zero = ui.add_sized(drag_size, egui::Button::new("Reset"));

                if zero.clicked() {
                    *pos = Vec3::ZERO;
                    *changed = true;
                }

                if x.changed() || y.changed() || z.changed() {
                    pos.x = pos_x;
                    pos.y = pos_y;
                    pos.z = pos_z;
                    *changed = true;
                }
            });
    });
}

fn display_scale_ui(ui: &mut egui::Ui, scale: &mut Vec3, changed: &mut bool, drag_size: [f32; 2]) {
    let spacing = crate::UI_CONFIG.large_spacing;
    ui.horizontal(|ui| {
        let label_width = (ui.available_width() / 5.) + spacing;
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(label_width, drag_size[1]), egui::Sense::hover());
        ui.painter().text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            "Scale:",
            egui::FontId::default(),
            ui.visuals().text_color(),
        );

        egui::Grid::new("scale_grid")
            .num_columns(4)
            .spacing([1.0, 0.0])
            .striped(true)
            .show(ui, |ui| {
                let mut scale_x = scale.x;
                let mut scale_y = scale.y;
                let mut scale_z = scale.z;

                let x = ui.add_sized(
                    drag_size,
                    egui::DragValue::new(&mut scale_x)
                        .speed(0.01)
                        .fixed_decimals(2),
                );
                x.context_menu(|ui| {
                    if ui.button("Reset").clicked() {
                        scale_x = 1.0;
                        scale.x = scale_x;
                        *changed = true;
                        ui.close();
                    }
                });

                let y = ui.add_sized(
                    drag_size,
                    egui::DragValue::new(&mut scale_y)
                        .speed(0.01)
                        .fixed_decimals(2),
                );
                y.context_menu(|ui| {
                    if ui.button("Reset").clicked() {
                        scale_y = 1.0;
                        scale.y = scale_y;
                        *changed = true;
                        ui.close();
                    }
                });

                let z = ui.add_sized(
                    drag_size,
                    egui::DragValue::new(&mut scale_z)
                        .speed(0.01)
                        .fixed_decimals(2),
                );
                z.context_menu(|ui| {
                    if ui.button("Reset").clicked() {
                        scale_z = 1.0;
                        scale.z = scale_z;
                        *changed = true;
                        ui.close();
                    }
                });

                let reset = ui.add_sized(drag_size, egui::Button::new("Reset"));

                if reset.clicked() {
                    *scale = Vec3::ONE;
                    *changed = true;
                }

                if x.changed() || y.changed() || z.changed() {
                    scale.x = scale_x;
                    scale.y = scale_y;
                    scale.z = scale_z;
                    *changed = true;
                }
            });
    });
}

//

fn clamp_angle_360(angle: f32) -> f32 {
    let mut a = angle % 360.0;
    if a > 180.0 {
        a -= 360.0;
    } else if a < -180.0 {
        a += 360.0;
    }
    a
}

fn closest_angle(old: f32, new: f32) -> f32 {
    let delta = (new - old + 180.0) % 360.0 - 180.0;
    old + delta
}

fn normalize_angle_visual(angle: f32) -> f32 {
    let a = angle % 360.0;
    if a.abs() < 0.001
        || (a - 180.0).abs() < 0.001
        || (a + 180.0).abs() < 0.001
        || a.abs() > 359.999
    {
        0.0
    } else {
        a
    }
}

fn normalize_euler_visual(euler: Vec3) -> Vec3 {
    Vec3::new(
        normalize_angle_visual(euler.x),
        normalize_angle_visual(euler.y),
        normalize_angle_visual(euler.z),
    )
}
