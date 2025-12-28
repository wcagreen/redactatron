use eframe::egui;
use std::path::PathBuf;

pub fn render_word_doc(ui: &mut egui::Ui, file_path: &PathBuf) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.label("📝 Word Document");
        ui.label(file_path.display().to_string());
        ui.add_space(20.0);
        ui.label("Word document rendering will be implemented here");
        ui.label("May need to convert to PDF first for rendering");
    });
}
