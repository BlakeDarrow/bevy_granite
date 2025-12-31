use super::Empty;
use crate::GraniteType;
use bevy_egui::egui;

impl Empty {
    /// Function to edit self's data via UI side panel
    /// In this case empty needs no data, so always returns false
    pub fn edit_via_ui(
        &mut self,
        ui: &mut egui::Ui,
        // Small, Large, Normal
        spacing: (f32, f32, f32),
    ) -> bool {
        let large_spacing = spacing.1;
        ui.label(egui::RichText::new(self.type_name()).italics());
        ui.add_space(large_spacing);
        
        let mut changed = false;
        
        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.hide_children, "Hide Children").changed() {
                changed = true;
            }
        });
        
        ui.add_space(large_spacing);
        changed
    }
}
