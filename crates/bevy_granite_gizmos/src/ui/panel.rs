use bevy::{
    ecs::{query::With, system::Query},
    prelude::ResMut,
};
use bevy_egui::{egui, EguiContexts};
use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};

use crate::{
    gizmos::{GizmoConfig, GizmoMode, GizmoSnap, GizmoType, Gizmos, NewGizmoConfig, NewGizmoType},
    ActiveSelection,
};

pub fn editor_gizmos_ui(
    mut contexts: EguiContexts,
    mut selected_option: ResMut<NewGizmoType>,
    mut gizmo_snap: ResMut<GizmoSnap>,
    mut config: ResMut<NewGizmoConfig>,
    selected_entity: Query<&Gizmos, With<ActiveSelection>>,
    mut gizmos: Query<&mut GizmoConfig>,
) {
    let small_spacing = 1.;
    let spacing = 4.;
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
                    ui.set_max_width(100.);
                    changed |= ui
                        .radio_value(&mut active, GizmoType::Pointer, "Pointer")
                        .changed();
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
