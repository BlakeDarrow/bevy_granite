use crate::GraniteType;

use super::OBJ;
use bevy_egui::egui;
use native_dialog::FileDialog;

impl OBJ {
    pub fn edit_via_ui(&mut self, ui: &mut egui::Ui, spacing: (f32, f32, f32)) -> bool {
        let large_spacing = spacing.1;
        let spacing_val = spacing.0;
        let small_spacing = spacing_val / 2.0;
        
        ui.label(egui::RichText::new(self.type_name()).italics());
        ui.add_space(large_spacing);
        
        // Path editing
        let mut changed = false;
        ui.label("OBJ Path:");
        ui.add_space(spacing_val);
        
        ui.horizontal(|ui| {
            ui.set_max_width(ui.available_width() - large_spacing * 3.);

            let mut path_string = self.mesh_path.to_string();
            if ui.text_edit_singleline(&mut path_string).changed() {
                self.mesh_path = path_string.into();
                changed = true;
            }

            ui.spacing_mut().button_padding = egui::Vec2::new(2.0, 2.0);
            if ui.button("📁").clicked() {
                let current_dir = std::env::current_dir().unwrap();
                let assets_dir = current_dir.join("assets");
                let models_path = assets_dir.join("models");

                // Use models dir if it exists or can be created, otherwise use current dir
                let dialog_path =
                    if models_path.exists() || std::fs::create_dir_all(&models_path).is_ok() {
                        models_path
                    } else {
                        current_dir
                    };

                if let Ok(Some(path)) = FileDialog::new()
                    .add_filter("OBJ Files", &["obj"])
                    .set_location(&dialog_path)
                    .show_open_single_file()
                {
                    let relative_path = if let Ok(rel_path) = path.strip_prefix(&assets_dir) {
                        rel_path.to_string_lossy().to_string().replace("\\", "/")
                    } else {
                        path.to_string_lossy().to_string()
                    };
                    self.mesh_path = relative_path.into();
                    changed = true;
                }
            }

            ui.add_space(small_spacing);
        });
        
        ui.add_space(large_spacing);
        
        let reload_clicked = ui.button("Reload OBJ").clicked();
        if reload_clicked {
            self.reload_requested = true;
        }
        
        reload_clicked || changed
    }
}
