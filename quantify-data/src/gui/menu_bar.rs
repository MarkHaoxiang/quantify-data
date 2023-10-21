/// Build the menu bar
pub fn add_contents(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        toggle_visual_mode(ui);    
        ui.separator();
    });
}

/// A Toggle between light and dark mode
pub fn toggle_visual_mode(ui: &mut egui::Ui) {
    let style: egui::Style = (*ui.ctx().style()).clone();
    let new_visuals: Option<egui::Visuals> = style.visuals.light_dark_small_toggle_button(ui);
    if let Some(visuals) = new_visuals {
        ui.ctx().set_visuals(visuals);
    }
}