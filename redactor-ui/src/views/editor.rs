use eframe::egui;
use egui::StrokeKind;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct EditorPage {
    files: Vec<PathBuf>,
    current_file_index: usize,
    search_query: String,
    redaction_areas: Vec<RedactionArea>,
    texture_cache: HashMap<PathBuf, egui::TextureHandle>,
    // Image viewing state
    zoom: f32,
    pan_offset: egui::Vec2,
    drag_start: Option<egui::Pos2>,
    redaction_mode: bool,
    hovered_redaction: Option<usize>,
    pending_delete: Option<usize>,
    // New fields for export feedback
    show_no_redactions_dialog: bool,
    show_export_success_dialog: bool,
    last_exported_filename: String,
    // Redactor go home
    return_to_home: bool,
}

#[derive(Clone, Debug)]
struct RedactionArea {
    // Store in normalized coordinates (0.0 to 1.0)
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    file_path: PathBuf,
}

impl EditorPage {
    pub fn new(files: Vec<PathBuf>) -> Self {
        Self {
            files,
            current_file_index: 0,
            search_query: String::new(),
            redaction_areas: Vec::new(),
            texture_cache: HashMap::new(),
            zoom: 1.0,
            pan_offset: egui::Vec2::ZERO,
            drag_start: None,
            redaction_mode: false,
            hovered_redaction: None,
            pending_delete: None,
            show_no_redactions_dialog: false,
            show_export_success_dialog: false,
            last_exported_filename: String::new(),
            return_to_home: false,
        }
    }

    // Returns true if all files have been processed and should return to home
    pub fn update(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Left sidebar with file list and search
        egui::SidePanel::left("sidebar")
            .min_width(250.0)
            .show_inside(ui, |ui| {
                self.render_sidebar(ui);
            });

        // Main content area - document viewer
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.render_document_viewer(ui, ctx);
        });

        // Show no redactions dialog
        if self.show_no_redactions_dialog {
            egui::Window::new("No Redactions")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("⚠️ No redactions detected for this file.");
                    ui.add_space(8.0);
                    ui.label("Please mark areas to redact before exporting.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("OK").clicked() {
                                self.show_no_redactions_dialog = false;
                            }
                        });
                    });
                });
        }

        // Show export success dialog
        if self.show_export_success_dialog {
            egui::Window::new("Export Successful")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("✅ File exported successfully!");
                    ui.add_space(8.0);
                    ui.label(format!("Saved as: {}", self.last_exported_filename));
                    ui.add_space(8.0);
                    ui.label("The file has been removed from the editor.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("OK").clicked() {
                                self.show_export_success_dialog = false;

                                // Check if files is not empty before trying to access it
                                if !self.files.is_empty() {
                                    // NOW remove the file after the user clicks OK
                                    let current_file = self.files[self.current_file_index].clone();

                                    // Remove redactions for this file
                                    self.redaction_areas
                                        .retain(|r| &r.file_path != &current_file);

                                    // Remove texture from cache
                                    self.texture_cache.remove(&current_file);

                                    // Remove file from list
                                    self.files.remove(self.current_file_index);

                                    // Adjust current index if needed
                                    if self.current_file_index >= self.files.len()
                                        && !self.files.is_empty()
                                    {
                                        self.current_file_index = self.files.len() - 1;
                                    }

                                    // Reset view state
                                    self.zoom = 1.0;
                                    self.pan_offset = egui::Vec2::ZERO;
                                    self.drag_start = None;
                                    self.redaction_mode = false;
                                }

                                // Check if we should return to home
                                if self.files.is_empty() {
                                    self.return_to_home = true;
                                }
                            }
                        });
                    });
                });
        }
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("Files");
        ui.separator();

        if self.return_to_home {
            ui.label("Nothing here Chief! Time to go back home.");
            return;
        }

        // File list
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, file) in self.files.iter().enumerate() {
                let file_name = file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");

                let is_selected = idx == self.current_file_index;

                if ui.selectable_label(is_selected, file_name).clicked() {
                    self.current_file_index = idx;
                    // Reset view state when switching files
                    self.zoom = 1.0;
                    self.pan_offset = egui::Vec2::ZERO;
                    self.drag_start = None;
                }
            }
        });

        ui.separator();
        ui.add_space(10.0);

        // Search section
        ui.heading("Search & Redact");
        ui.separator();

        ui.label("Search for keywords:");
        let response = ui.text_edit_singleline(&mut self.search_query);

        if response.changed() || ui.button("🔍 Search").clicked() {
            if !self.search_query.is_empty() {
                println!("Searching for: {}", self.search_query);
                // TODO: Implement search functionality
            }
        }

        ui.add_space(10.0);

        if ui.button("Clear Search").clicked() {
            self.search_query.clear();
        }

        ui.separator();
        ui.add_space(10.0);

        // Redaction controls
        ui.heading("Redaction");

        let current_file = &self.files[self.current_file_index];
        let redactions_for_current = self
            .redaction_areas
            .iter()
            .filter(|r| &r.file_path == current_file)
            .count();

        ui.label(format!("Areas marked: {}", redactions_for_current));

        ui.add_space(10.0);

        // Redaction mode toggle button
        let mode_button = if self.redaction_mode {
            egui::Button::new("✏️ Redaction Mode: ON").fill(egui::Color32::from_rgb(220, 50, 50))
        } else {
            egui::Button::new("✏️ Redaction Mode: OFF").fill(egui::Color32::from_rgb(60, 60, 65))
        };

        if ui.add(mode_button).clicked() {
            self.redaction_mode = !self.redaction_mode;
            self.drag_start = None;
        }

        if self.redaction_mode {
            ui.colored_label(
                egui::Color32::LIGHT_YELLOW,
                "Left-click drag to create redaction",
            );
        } else {
            ui.label("Left-click drag to pan image");
        }

        ui.add_space(10.0);

        if ui.button("Clear Current File").clicked() {
            self.redaction_areas
                .retain(|r| &r.file_path != current_file);
        }

        if ui.button("Clear All Redactions").clicked() {
            self.redaction_areas.clear();
        }

        ui.add_space(10.0);

        if ui.button("💾 Export Redacted Document").clicked() {
            self.export_redacted_file();
        }
    }

    fn render_document_viewer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.return_to_home {
            ui.centered_and_justified(|ui| {
                ui.label("Well done! All files have been processed. Time to go back home.");
            });
            return;
        }

        // Extract values before closures to avoid borrow issues
        let current_file = self.files[self.current_file_index].clone();
        let file_name = current_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let current_index = self.current_file_index;
        let total_files = self.files.len();

        ui.vertical(|ui| {
            // Document header
            ui.horizontal(|ui| {
                ui.heading(format!("Viewing: {}", file_name));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("File {} of {}", current_index + 1, total_files));
                });
            });
            ui.separator();
        });

        // Document viewer area (outside the vertical closure to avoid borrow issues)
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.render_document_content(ui, &current_file, ctx);
            });
    }

    fn render_document_content(
        &mut self,
        ui: &mut egui::Ui,
        file_path: &PathBuf,
        ctx: &egui::Context,
    ) {
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "pdf" => self.render_pdf(ui, file_path),
            "jpg" | "jpeg" | "png" | "gif" | "webp" => self.render_image(ui, file_path, ctx),
            "doc" | "docx" => self.render_word_doc(ui, file_path),
            _ => {
                ui.label("Unsupported file format");
                ui.label(format!("Extension: {}", extension));
            }
        }
    }

    fn render_pdf(&mut self, ui: &mut egui::Ui, file_path: &PathBuf) {
        // TODO: Implement PDF rendering using pdfium-render
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label("📄 PDF Document");
            ui.label(file_path.display().to_string());
            ui.add_space(20.0);
            ui.label("PDF rendering will be implemented here");
            ui.label("Using pdfium-render to display pages");

            // Placeholder for PDF content
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(600.0, 800.0), egui::Sense::click_and_drag());

            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(250, 250, 250));

            // Handle redaction area selection
            if response.dragged() {
                if let Some(start_pos) = response.interact_pointer_pos() {
                    // TODO: Track drag to create redaction area
                    println!("Dragging at: {:?}", start_pos);
                }
            }
        });
    }

    fn render_image(&mut self, ui: &mut egui::Ui, file_path: &PathBuf, ctx: &egui::Context) {
        // Check if image is already in cache
        if !self.texture_cache.contains_key(file_path) {
            // Load image from file
            match image::open(file_path) {
                Ok(img) => {
                    // Convert to RGBA8
                    let img_rgba = img.to_rgba8();
                    let size = [img_rgba.width() as _, img_rgba.height() as _];
                    let pixels = img_rgba.as_flat_samples();

                    // Create egui ColorImage
                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                    // Load into texture
                    let texture = ctx.load_texture(
                        file_path.to_string_lossy(),
                        color_image,
                        Default::default(),
                    );

                    self.texture_cache.insert(file_path.clone(), texture);
                }
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Failed to load image: {}", e));
                    return;
                }
            }
        }

        // Display the cached texture
        if let Some(texture) = self.texture_cache.get(file_path) {
            let original_size = texture.size_vec2();

            ui.vertical(|ui| {
                // Zoom controls
                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    if ui.button("➖").clicked() {
                        self.zoom = (self.zoom - 0.1).max(0.1);
                    }
                    ui.label(format!("{:.0}%", self.zoom * 100.0));
                    if ui.button("➕").clicked() {
                        self.zoom = (self.zoom + 0.1).min(5.0);
                    }
                    if ui.button("Reset View").clicked() {
                        self.zoom = 1.0;
                        self.pan_offset = egui::Vec2::ZERO;
                    }
                    ui.separator();
                    if self.redaction_mode {
                        ui.colored_label(
                            egui::Color32::LIGHT_RED,
                            "🖍️ REDACTION MODE - Drag to create boxes",
                        );
                    } else {
                        ui.label("🖐️ PAN MODE - Drag to move image");
                    }
                });

                ui.separator();

                // Image display area with scrolling
                let scaled_size = original_size * self.zoom;

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(scaled_size, egui::Sense::click_and_drag());

                        // Handle scroll wheel zoom
                        if response.hovered() {
                            let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
                            if scroll_delta != 0.0 {
                                let zoom_delta = scroll_delta * 0.001;
                                self.zoom = (self.zoom + zoom_delta).clamp(0.1, 5.0);
                            }
                        }

                        let image_rect =
                            egui::Rect::from_min_size(rect.min + self.pan_offset, scaled_size);

                        // Draw the image
                        ui.painter().image(
                            texture.id(),
                            image_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );

                        // Draw existing redaction areas for this file and detect hover
                        self.hovered_redaction = None;
                        for (idx, redaction) in self.redaction_areas.iter().enumerate() {
                            if &redaction.file_path == file_path {
                                let redact_rect = egui::Rect::from_min_size(
                                    image_rect.min
                                        + egui::vec2(
                                            redaction.x * image_rect.width(),
                                            redaction.y * image_rect.height(),
                                        ),
                                    egui::vec2(
                                        redaction.width * image_rect.width(),
                                        redaction.height * image_rect.height(),
                                    ),
                                );

                                // Check if pointer is over this redaction
                                let mut is_hovered = false;
                                if let Some(pointer_pos) = response.interact_pointer_pos() {
                                    if redact_rect.contains(pointer_pos) {
                                        self.hovered_redaction = Some(idx);
                                        is_hovered = true;
                                    }
                                }

                                // Draw black rectangle
                                ui.painter()
                                    .rect_filled(redact_rect, 0.0, egui::Color32::BLACK);

                                // Draw border (yellow when hovered, red otherwise)
                                let border_color = if is_hovered {
                                    egui::Color32::YELLOW
                                } else {
                                    egui::Color32::RED
                                };
                                let thickness = if is_hovered { 3.0 } else { 2.0 };

                                ui.painter().rect_stroke(
                                    redact_rect,
                                    0.0,
                                    egui::Stroke::new(thickness, border_color),
                                    StrokeKind::Outside,
                                );

                                // Right-click to mark for deletion (confirmation shown later)
                                if is_hovered && response.clicked_by(egui::PointerButton::Secondary)
                                {
                                    self.pending_delete = Some(idx);
                                }
                            }
                        }

                        // If a deletion was requested, show confirmation window
                        if let Some(del_idx) = self.pending_delete {
                            // Ensure index is still valid
                            if del_idx < self.redaction_areas.len() {
                                egui::Window::new("Confirm Delete")
                                    .collapsible(false)
                                    .resizable(false)
                                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                                    .show(ctx, |ui| {
                                        ui.label("Are you sure you want to delete this redaction?");
                                        ui.add_space(8.0);
                                        ui.horizontal(|ui| {
                                            if ui.button("Delete").clicked() {
                                                if del_idx < self.redaction_areas.len() {
                                                    self.redaction_areas.remove(del_idx);
                                                }
                                                self.pending_delete = None;
                                                self.hovered_redaction = None;
                                            }
                                            if ui.button("Cancel").clicked() {
                                                self.pending_delete = None;
                                            }
                                        });
                                    });
                            } else {
                                self.pending_delete = None;
                            }
                        }

                        if !self.redaction_mode && response.dragged_by(egui::PointerButton::Primary)
                        {
                            self.pan_offset += response.drag_delta();
                        }

                        // Handle redaction drawing with left mouse button

                        if self.redaction_mode {
                            if response.drag_started_by(egui::PointerButton::Primary) {
                                self.drag_start = response.interact_pointer_pos();
                            }

                            if response.dragged_by(egui::PointerButton::Primary) {
                                if let (Some(start), Some(current)) =
                                    (self.drag_start, response.interact_pointer_pos())
                                {
                                    let preview_rect = egui::Rect::from_two_pos(start, current)
                                        .intersect(image_rect);
                                    ui.painter().rect_stroke(
                                        preview_rect,
                                        0.0,
                                        egui::Stroke::new(2.0, egui::Color32::RED),
                                        StrokeKind::Outside,
                                    );
                                }
                            }

                            if response.drag_stopped_by(egui::PointerButton::Primary) {
                                if let (Some(start), Some(end)) =
                                    (self.drag_start, response.interact_pointer_pos())
                                {
                                    let normalized_start = egui::pos2(
                                        ((start.x - image_rect.min.x) / image_rect.width())
                                            .clamp(0.0, 1.0),
                                        ((start.y - image_rect.min.y) / image_rect.height())
                                            .clamp(0.0, 1.0),
                                    );
                                    let normalized_end = egui::pos2(
                                        ((end.x - image_rect.min.x) / image_rect.width())
                                            .clamp(0.0, 1.0),
                                        ((end.y - image_rect.min.y) / image_rect.height())
                                            .clamp(0.0, 1.0),
                                    );

                                    let min_x = normalized_start.x.min(normalized_end.x);
                                    let min_y = normalized_start.y.min(normalized_end.y);
                                    let width = (normalized_end.x - normalized_start.x).abs();
                                    let height = (normalized_end.y - normalized_start.y).abs();

                                    if width > 0.01 && height > 0.01 {
                                        self.redaction_areas.push(RedactionArea {
                                            x: min_x,
                                            y: min_y,
                                            width,
                                            height,
                                            file_path: file_path.clone(),
                                        });
                                    }
                                }

                                self.drag_start = None;
                            }
                        }
                    });

                ui.add_space(10.0);
                ui.label(format!(
                    "Original size: {}x{} pixels",
                    original_size.x, original_size.y
                ));
            });
        }
    }

    fn render_word_doc(&mut self, ui: &mut egui::Ui, file_path: &PathBuf) {
        // TODO: Implement Word document rendering
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label("📝 Word Document");
            ui.label(file_path.display().to_string());
            ui.add_space(20.0);
            ui.label("Word document rendering will be implemented here");
            ui.label("May need to convert to PDF first for rendering");
        });
    }

    fn redact_and_export_image(&mut self, current_file: &PathBuf) -> bool {
        // Load the original image
        let img = match image::open(current_file) {
            Ok(img) => img,
            Err(e) => {
                println!("Failed to load image for export: {}", e);
                return false;
            }
        };

        let mut img_rgba = img.to_rgba8();
        let (width, height) = img_rgba.dimensions();

        // Get redactions for this file
        let redactions: Vec<_> = self
            .redaction_areas
            .iter()
            .filter(|r| &r.file_path == current_file)
            .collect();

        // Apply redactions by painting black rectangles
        for redaction in redactions {
            let x = (redaction.x * width as f32) as u32;
            let y = (redaction.y * height as f32) as u32;
            let w = (redaction.width * width as f32) as u32;
            let h = (redaction.height * height as f32) as u32;

            // Paint black pixels
            for py in y..(y + h).min(height) {
                for px in x..(x + w).min(width) {
                    img_rgba.put_pixel(px, py, image::Rgba([0, 0, 0, 255]));
                }
            }
        }

        // Open save dialog
        if let Some(save_path) = rfd::FileDialog::new()
            .set_file_name(format!(
                "{}_redacted.png",
                current_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("image")
            ))
            .add_filter("PNG Image", &["png"])
            .save_file()
        {
            match img_rgba.save(&save_path) {
                Ok(_) => {
                    println!("Redacted image saved to: {}", save_path.display());
                    self.last_exported_filename = save_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file")
                        .to_string();
                    return true;
                }
                Err(e) => {
                    println!("Failed to save redacted image: {}", e);
                    return false;
                }
            }
        }

        false
    }

    fn export_redacted_file(&mut self) {
        let current_file = self.files[self.current_file_index].clone();

        // Check if there are any redactions for this file
        let has_redactions = self
            .redaction_areas
            .iter()
            .any(|r| &r.file_path == &current_file);

        if !has_redactions {
            self.show_no_redactions_dialog = true;
            return;
        }

        // Check if current file is an image
        let extension = current_file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let export_success = if matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp") {
            self.redact_and_export_image(&current_file)
        } else {
            // For other file types, show not implemented message
            println!("Export not yet implemented for this file type");
            false
        };

        if export_success {
            // DON'T remove the file immediately - just show the success dialog
            // The file will be removed when the user clicks OK on the dialog
            self.show_export_success_dialog = true;
        }
    }
}
