use eframe::egui;
use std::path::PathBuf;

pub struct HomePage {
    uploaded_files: Vec<PathBuf>,
    drag_hover: bool,
}

impl Default for HomePage {
    fn default() -> Self {
        Self {
            uploaded_files: Vec::new(),
            drag_hover: false,
        }
    }
}

impl HomePage {
    /// Update the home page UI
    /// Returns Some(files) when user clicks Submit, None otherwise
    pub fn update(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> Option<Vec<PathBuf>> {
        let mut submitted_files = None;

        ui.vertical(|ui| {
            ui.label(egui::RichText::new("📄 File Redactor Application").size(18.0).strong());
            ui.label("Redact sensitive information from documents");
            ui.horizontal(|ui| {
                // TODO replace with actual links
                ui.hyperlink_to("Documentation", "https://github.com/wcagreen/rusty-redactor");
                ui.separator();
                ui.hyperlink_to("GitHub", "https://github.com/wcagreen/rusty-redactor");
            });
            ui.separator();

            // Drag and drop zone
            let drop_area = egui::Frame::NONE
                .fill(if self.drag_hover {
                    egui::Color32::from_rgb(100, 149, 237)
                } else {
                    egui::Color32::from_rgb(240, 240, 240)
                })
                .inner_margin(egui::Margin {
                    left: 20,
                    right: 20,
                    top: 40,
                    bottom: 40,
                });

            drop_area.show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.heading("📁 Drag and Drop Files Here");
                    
                    // Browse button
                    if ui.button("📂 Click to Browse").clicked() {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter("Documents", &["pdf", "jpg", "jpeg", "png", "gif", "webp", "doc", "docx"])
                            .add_filter("All Files", &["*"])
                            .pick_files()
                        {
                            for path in paths {
                                println!("Selected file: {}", path.display());
                                self.uploaded_files.push(path);
                            }
                        }
                    }
                    ui.add_space(20.0);
                });
            });

            ui.separator();
            
            // Check for dragged files
            self.drag_hover = ctx.input(|i| i.raw.hovered_files.len() > 0);

            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    println!("Files dropped: {} files", i.raw.dropped_files.len());
                    for file in &i.raw.dropped_files {
                        if let Some(path) = &file.path {
                            println!("Dropped file: {}", path.display());
                            self.uploaded_files.push(path.clone());
                        }
                    }
                }
            });

            ui.separator();

            // Display uploaded files
            if !self.uploaded_files.is_empty() {
                ui.label(egui::RichText::new("📤 Uploaded Files:").size(18.0).strong());

                // Simple file list
                for file in &self.uploaded_files {
                    ui.horizontal(|ui| {
                        ui.label("•");
                        ui.label(file.display().to_string());
                    });
                }

                ui.add_space(6.0);

                // Action row: right-aligned Clear and Submit buttons
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let has_files = !self.uploaded_files.is_empty();

                        // Submit (primary)
                        let submit_btn = egui::Button::new(
                            egui::RichText::new("Submit").size(16.0).strong(),
                        )
                        .fill(egui::Color32::from_rgb(10, 132, 255))
                        .min_size(egui::vec2(120.0, 34.0));

                        if ui.add_enabled(has_files, submit_btn).clicked() {
                            println!("Submitting {} files for processing", self.uploaded_files.len());
                            submitted_files = Some(self.uploaded_files.clone());
                            self.uploaded_files.clear();
                        }

                        ui.add_space(8.0);

                        // Clear (secondary)
                        let clear_btn = egui::Button::new(egui::RichText::new("Clear Files").size(14.0))
                            .fill(egui::Color32::from_rgb(80, 82, 85))
                            .min_size(egui::vec2(110.0, 34.0));

                        if ui.add_enabled(has_files, clear_btn).clicked() {
                            self.uploaded_files.clear();
                        }
                    });
                });
            }
        });

        submitted_files
    }
}