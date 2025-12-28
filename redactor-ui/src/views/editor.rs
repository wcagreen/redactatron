use eframe::egui;
use egui::StrokeKind;
use redactor_core::processors::pdf::{PdfEngine, SearchResult};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct EditorPage {
    files: Vec<PathBuf>,
    current_file_index: usize,
    search_query: String,
    redaction_areas: Vec<RedactionArea>,
    texture_cache: HashMap<(PathBuf, u16), egui::TextureHandle>, // (path, page_index)
    // Image viewing state
    zoom: f32,
    pan_offset: egui::Vec2,
    drag_start: Option<egui::Pos2>,
    redaction_mode: bool,
    hovered_redaction: Option<usize>,
    pending_delete: Option<usize>,
    // PDF state
    pdf_engine: Option<PdfEngine>,
    current_pdf_page: u16,
    pdf_page_count: u16,
    search_results: Vec<SearchResult>,
    // Export feedback
    show_no_redactions_dialog: bool,
    show_export_success_dialog: bool,
    last_exported_filename: String,
    return_to_home: bool,
}

#[derive(Clone, Debug)]
struct RedactionArea {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    file_path: PathBuf,
    page_index: Option<u16>, // For PDFs, None for images
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
            pdf_engine: None,
            current_pdf_page: 0,
            pdf_page_count: 0,
            search_results: Vec::new(),
            show_no_redactions_dialog: false,
            show_export_success_dialog: false,
            last_exported_filename: String::new(),
            return_to_home: false,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .min_width(250.0)
            .show_inside(ui, |ui| {
                self.render_sidebar(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.render_document_viewer(ui, ctx);
        });

        if !self.search_results.is_empty() {
            egui::SidePanel::right("search_results_panel")
                .min_width(320.0)
                .resizable(true)
                .show_inside(ui, |ui| {
                    self.render_search_results_panel(ui);
                });
        }

        self.show_dialogs(ctx);
    }

    fn show_dialogs(&mut self, ctx: &egui::Context) {
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
                    if ui.button("OK").clicked() {
                        self.show_no_redactions_dialog = false;
                    }
                });
        }

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
                    if ui.button("OK").clicked() {
                        self.show_export_success_dialog = false;
                        self.remove_current_file();
                    }
                });
        }
    }

    fn remove_current_file(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let current_file = self.files[self.current_file_index].clone();

        // Clear PDF engine if current file is PDF
        if self.is_pdf(&current_file) {
            self.pdf_engine = None;
            self.search_results.clear();
        }

        self.redaction_areas
            .retain(|r| &r.file_path != &current_file);

        // Clear texture cache for this file
        self.texture_cache
            .retain(|(path, _), _| path != &current_file);

        self.files.remove(self.current_file_index);

        if self.current_file_index >= self.files.len() && !self.files.is_empty() {
            self.current_file_index = self.files.len() - 1;
        }

        self.reset_view_state();

        if self.files.is_empty() {
            self.return_to_home = true;
        } else {
            self.load_current_file();
        }
    }

    fn reset_view_state(&mut self) {
        self.zoom = 1.0;
        self.pan_offset = egui::Vec2::ZERO;
        self.drag_start = None;
        self.redaction_mode = false;
        self.current_pdf_page = 0;
    }

    fn load_current_file(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let current_file = &self.files[self.current_file_index];

        if self.is_pdf(current_file) {
            let mut engine = PdfEngine::new();
            if let Ok(_) = engine.load_file(&current_file.to_string_lossy()) {
                self.pdf_page_count = engine.page_count();
                self.current_pdf_page = 0;
                self.pdf_engine = Some(engine);
                self.search_results.clear();
            }
        }
    }

    fn is_pdf(&self, file_path: &PathBuf) -> bool {
        file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase() == "pdf")
            .unwrap_or(false)
    }

    fn render_search_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Search Results");
        ui.separator();

        ui.label(format!("Found {} matches", self.search_results.len()));
        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .id_salt("right_search_results_scroll_area")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (idx, result) in self.search_results.iter().enumerate() {
                    let preview = if result.text.len() > 40 {
                        format!("{}...", &result.text[..40])
                    } else {
                        result.text.clone()
                    };

                    let label = format!(
                        "{}. Page {} — \"{}\"",
                        idx + 1,
                        result.page_index + 1,
                        preview
                    );

                    if ui
                        .selectable_label(result.page_index == self.current_pdf_page, label)
                        .clicked()
                    {
                        self.current_pdf_page = result.page_index;
                        self.pan_offset = egui::Vec2::ZERO;
                    }

                    ui.add_space(4.0);
                }
            });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("Files");
        ui.separator();

        if self.return_to_home {
            ui.label("Nothing here Chief! Time to go back home.");
            return;
        }

        let mut clicked_index = None;

        egui::ScrollArea::vertical()
            .id_salt("file_list_scroll")
            .show(ui, |ui| {
                for (idx, file) in self.files.iter().enumerate() {
                    let file_name = file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");

                    let is_selected = idx == self.current_file_index;

                    if ui.selectable_label(is_selected, file_name).clicked() {
                        clicked_index = Some(idx);
                    }
                }
            });

        if let Some(idx) = clicked_index {
            self.current_file_index = idx;
            self.reset_view_state();
            self.load_current_file();
        }

        ui.separator();
        ui.add_space(10.0);

        // Search section
        ui.heading("Search & Redact");
        ui.separator();

        ui.label("Search for keywords:");
        let response = ui.text_edit_singleline(&mut self.search_query);

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
            || ui.button("🔍 Search").clicked()
        {
            if !self.search_query.is_empty() {
                self.perform_search();
            }
        }

        ui.add_space(10.0);

        if ui.button("Clear Search").clicked() {
            self.search_query.clear();
            self.search_results.clear();
        }

        ui.separator();
        ui.add_space(10.0);

        // Redaction controls
        ui.heading("Redaction");

        let current_file = &self.files[self.current_file_index];
        let redactions_for_current = if self.is_pdf(current_file) {
            self.redaction_areas
                .iter()
                .filter(|r| {
                    &r.file_path == current_file && r.page_index == Some(self.current_pdf_page)
                })
                .count()
        } else {
            self.redaction_areas
                .iter()
                .filter(|r| &r.file_path == current_file)
                .count()
        };

        ui.label(format!("Areas marked: {}", redactions_for_current));
        ui.add_space(10.0);

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
            ui.label("Left-click drag to pan");
        }

        ui.add_space(10.0);

        if ui.button("Clear Current Page").clicked() {
            if self.is_pdf(current_file) {
                self.redaction_areas.retain(|r| {
                    &r.file_path != current_file || r.page_index != Some(self.current_pdf_page)
                });
            } else {
                self.redaction_areas
                    .retain(|r| &r.file_path != current_file);
            }
        }

        if ui.button("Clear All Redactions").clicked() {
            self.redaction_areas.clear();
        }

        ui.add_space(10.0);

        if ui.button("💾 Export Redacted Document").clicked() {
            self.export_redacted_file();
        }
    }

    fn perform_search(&mut self) {
        self.search_results.clear();

        if let Some(engine) = &self.pdf_engine {
            match engine.search(&self.search_query) {
                Ok(results) => {
                    self.search_results = results;
                    println!("Found {} search results", self.search_results.len());
                }
                Err(e) => {
                    println!("Search error: {}", e);
                }
            }
        }
    }

    fn render_document_viewer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.return_to_home || self.files.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Well done! All files have been processed. Time to go back home.");
            });
            return;
        }

        let current_file = self.files[self.current_file_index].clone();
        let file_name = current_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Viewing: {}", file_name));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "File {} of {}",
                        self.current_file_index + 1,
                        self.files.len()
                    ));
                });
            });
            ui.separator();
        });

        egui::ScrollArea::both()
            .id_salt("render_document_scroll_area")
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
            "pdf" => self.render_pdf(ui, file_path, ctx),
            "jpg" | "jpeg" | "png" | "gif" | "webp" => self.render_image(ui, file_path, ctx),
            "doc" | "docx" => self.render_word_doc(ui, file_path),
            _ => {
                ui.label("Unsupported file format");
                ui.label(format!("Extension: {}", extension));
            }
        }
    }

    fn render_pdf(&mut self, ui: &mut egui::Ui, file_path: &PathBuf, ctx: &egui::Context) {
        // Initialize PDF engine if needed
        if self.pdf_engine.is_none() {
            let mut engine = PdfEngine::new();
            if let Ok(_) = engine.load_file(&file_path.to_string_lossy()) {
                self.pdf_page_count = engine.page_count();
                self.pdf_engine = Some(engine);
            } else {
                ui.colored_label(egui::Color32::RED, "Failed to load PDF");
                return;
            }
        }

        let page_count = self.pdf_page_count;

        ui.vertical(|ui| {
            // Page navigation and controls
            ui.horizontal(|ui| {
                ui.label("Page:");
                if ui.button("◀").clicked() && self.current_pdf_page > 0 {
                    self.current_pdf_page -= 1;
                    self.pan_offset = egui::Vec2::ZERO;
                }

                ui.label(format!("{} / {}", self.current_pdf_page + 1, page_count));

                if ui.button("▶").clicked() && self.current_pdf_page < page_count - 1 {
                    self.current_pdf_page += 1;
                    self.pan_offset = egui::Vec2::ZERO;
                }

                ui.separator();

                // Zoom controls
                ui.label("Zoom:");
                if ui.button("➖").clicked() {
                    self.zoom = (self.zoom - 0.1).max(0.5);
                }
                ui.label(format!("{:.0}%", self.zoom * 100.0));
                if ui.button("➕").clicked() {
                    self.zoom = (self.zoom + 0.1).min(3.0);
                }
                if ui.button("Reset").clicked() {
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
                    ui.label("🖐️ PAN MODE - Drag to move");
                }
            });

            ui.separator();

            // Render PDF page
            let cache_key = (file_path.clone(), self.current_pdf_page);

            if !self.texture_cache.contains_key(&cache_key) {
                if let Some(engine) = &self.pdf_engine {
                    match engine.render_page(self.current_pdf_page, 2.0) {
                        Ok(img) => {
                            let img_rgba = img.to_rgba8();
                            let size = [img_rgba.width() as _, img_rgba.height() as _];
                            let pixels = img_rgba.as_flat_samples();

                            let color_image =
                                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                            let texture = ctx.load_texture(
                                format!(
                                    "{}_{}",
                                    file_path.to_string_lossy(),
                                    self.current_pdf_page
                                ),
                                color_image,
                                Default::default(),
                            );

                            self.texture_cache.insert(cache_key.clone(), texture);
                        }
                        Err(e) => {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("Failed to render page: {}", e),
                            );
                            return;
                        }
                    }
                }
            }

            if let Some(texture) = self.texture_cache.get(&cache_key) {
                let texture_id = texture.id();
                let texture_size = texture.size_vec2();

                self.render_pdf_page_with_highlights(ui, texture_id, texture_size, file_path, ctx);
            }
        });
    }

    fn render_pdf_page_with_highlights(
        &mut self,
        ui: &mut egui::Ui,
        texture_id: egui::TextureId,
        texture_size: egui::Vec2,
        file_path: &PathBuf,
        ctx: &egui::Context,
    ) {
        let scaled_size = texture_size * self.zoom;

        let (rect, response) = ui.allocate_exact_size(scaled_size, egui::Sense::click_and_drag());

        // Handle zoom with scroll wheel
        if response.hovered() {
            let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom = (self.zoom + scroll_delta * 0.001).clamp(0.5, 3.0);
            }
        }

        let image_rect = egui::Rect::from_min_size(rect.min + self.pan_offset, scaled_size);

        // Draw the PDF page
        ui.painter().image(
            texture_id,
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );

        // Highlight search results
        self.draw_search_highlights(ui, &image_rect);

        // Draw redaction areas
        self.draw_redaction_areas(ui, file_path, &image_rect, &response, ctx);

        // Handle interaction
        self.handle_interaction(ui, file_path, &image_rect, &response);
    }

    fn draw_search_highlights(&self, ui: &mut egui::Ui, image_rect: &egui::Rect) {
        if let Some(engine) = &self.pdf_engine {
            for (idx, result) in self.search_results.iter().enumerate() {
                if result.page_index == self.current_pdf_page {
                    let rect_bounds = result.rect;

                    if let Ok((page_width, page_height)) =
                        engine.get_page_dimensions(self.current_pdf_page)
                    {
                        let page_width = page_width as f32;
                        let page_height = page_height as f32;

                        let width = rect_bounds.width().value as f32;
                        let height = rect_bounds.height().value as f32;

                        let x_norm = rect_bounds.left().value as f32 / page_width;
                        let y_norm =
                            1.0 - (rect_bounds.bottom().value as f32 + height) / page_height;
                        let width_norm = width / page_width;
                        let height_norm = height / page_height;

                        let highlight_rect = egui::Rect::from_min_size(
                            image_rect.min
                                + egui::vec2(
                                    x_norm * image_rect.width(),
                                    y_norm * image_rect.height(),
                                ),
                            egui::vec2(
                                width_norm * image_rect.width(),
                                height_norm * image_rect.height(),
                            ),
                        );
                        // Draw highlight
                        ui.painter().rect_filled(
                            highlight_rect,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 0, 100),
                        );

                        // Draw border
                        ui.painter().rect_stroke(
                            highlight_rect,
                            0.0,
                            egui::Stroke::new(1.5, egui::Color32::YELLOW),
                            egui::StrokeKind::Outside,
                        );

                        // Draw search text
                        let info_text = if result.text.len() > 50 {
                            format!("{}...", &result.text[..50])
                        } else {
                            result.text.clone()
                        };

                        let info_pos = if idx == 0 {
                            highlight_rect.left_top() + egui::vec2(0.0, -highlight_rect.height())
                        } else {
                            highlight_rect.right_top() + egui::vec2(5.0, 0.0)
                        };

                        ui.painter().text(
                            info_pos,
                            egui::Align2::LEFT_TOP,
                            info_text,
                            egui::TextStyle::Small.resolve(&ui.style()),
                            egui::Color32::WHITE,
                        );
                    }
                }
            }
        }
    }

    fn draw_redaction_areas(
        &mut self,
        ui: &mut egui::Ui,
        file_path: &PathBuf,
        image_rect: &egui::Rect,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        self.hovered_redaction = None;

        let current_page = if self.is_pdf(file_path) {
            Some(self.current_pdf_page)
        } else {
            None
        };

        // Draw redaction areas
        for (idx, redaction) in self.redaction_areas.iter().enumerate() {
            let should_draw =
                &redaction.file_path == file_path && redaction.page_index == current_page;

            if should_draw {
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

                let mut is_hovered = false;
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    if redact_rect.contains(pointer_pos) {
                        self.hovered_redaction = Some(idx);
                        is_hovered = true;
                    }
                }

                ui.painter()
                    .rect_filled(redact_rect, 0.0, egui::Color32::BLACK);

                let border_color = if is_hovered {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::RED
                };

                ui.painter().rect_stroke(
                    redact_rect,
                    0.0,
                    egui::Stroke::new(if is_hovered { 3.0 } else { 2.0 }, border_color),
                    StrokeKind::Outside,
                );

                if is_hovered && response.clicked_by(egui::PointerButton::Secondary) {
                    self.pending_delete = Some(idx);
                }
            }
        }

        // Show delete confirmation
        if let Some(del_idx) = self.pending_delete {
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
                                self.redaction_areas.remove(del_idx);
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
    }

    fn handle_interaction(
        &mut self,
        ui: &mut egui::Ui,
        file_path: &PathBuf,
        image_rect: &egui::Rect,
        response: &egui::Response,
    ) {
        if !self.redaction_mode && response.dragged_by(egui::PointerButton::Primary) {
            self.pan_offset += response.drag_delta();
        }

        if self.redaction_mode {
            if response.drag_started_by(egui::PointerButton::Primary) {
                self.drag_start = response.interact_pointer_pos();
            }

            if response.dragged_by(egui::PointerButton::Primary) {
                if let (Some(start), Some(current)) =
                    (self.drag_start, response.interact_pointer_pos())
                {
                    let preview_rect =
                        egui::Rect::from_two_pos(start, current).intersect(*image_rect);
                    ui.painter().rect_stroke(
                        preview_rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::RED),
                        StrokeKind::Outside,
                    );
                }
            }

            if response.drag_stopped_by(egui::PointerButton::Primary) {
                if let (Some(start), Some(end)) = (self.drag_start, response.interact_pointer_pos())
                {
                    self.create_redaction_area(start, end, image_rect, file_path);
                }
                self.drag_start = None;
            }
        }
    }

    fn create_redaction_area(
        &mut self,
        start: egui::Pos2,
        end: egui::Pos2,
        image_rect: &egui::Rect,
        file_path: &PathBuf,
    ) {
        let normalized_start = egui::pos2(
            ((start.x - image_rect.min.x) / image_rect.width()).clamp(0.0, 1.0),
            ((start.y - image_rect.min.y) / image_rect.height()).clamp(0.0, 1.0),
        );
        let normalized_end = egui::pos2(
            ((end.x - image_rect.min.x) / image_rect.width()).clamp(0.0, 1.0),
            ((end.y - image_rect.min.y) / image_rect.height()).clamp(0.0, 1.0),
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
                page_index: if self.is_pdf(file_path) {
                    Some(self.current_pdf_page)
                } else {
                    None
                },
            });
        }
    }

    fn render_image(&mut self, ui: &mut egui::Ui, file_path: &PathBuf, ctx: &egui::Context) {
        let cache_key = (file_path.clone(), 0);

        if !self.texture_cache.contains_key(&cache_key) {
            match image::open(file_path) {
                Ok(img) => {
                    let img_rgba = img.to_rgba8();
                    let size = [img_rgba.width() as _, img_rgba.height() as _];
                    let pixels = img_rgba.as_flat_samples();

                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                    let texture = ctx.load_texture(
                        file_path.to_string_lossy(),
                        color_image,
                        Default::default(),
                    );

                    self.texture_cache.insert(cache_key.clone(), texture);
                }
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Failed to load image: {}", e));
                    return;
                }
            }
        }

        if let Some(texture) = self.texture_cache.get(&cache_key) {
            let original_size = texture.size_vec2();
            let texture_id = texture.id();
            let scaled_size = original_size * self.zoom;

            ui.vertical(|ui| {
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

                let (rect, response) =
                    ui.allocate_exact_size(scaled_size, egui::Sense::click_and_drag());

                if response.hovered() {
                    let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
                    if scroll_delta != 0.0 {
                        self.zoom = (self.zoom + scroll_delta * 0.001).clamp(0.1, 5.0);
                    }
                }

                let image_rect = egui::Rect::from_min_size(rect.min + self.pan_offset, scaled_size);

                ui.painter().image(
                    texture_id,
                    image_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                self.draw_redaction_areas(ui, file_path, &image_rect, &response, ctx);
                self.handle_interaction(ui, file_path, &image_rect, &response);

                ui.add_space(10.0);
                ui.label(format!(
                    "Original size: {}x{} pixels",
                    original_size.x, original_size.y
                ));
            });
        }
    }

    fn render_word_doc(&mut self, ui: &mut egui::Ui, file_path: &PathBuf) {
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
        let img = match image::open(current_file) {
            Ok(img) => img,
            Err(e) => {
                println!("Failed to load image for export: {}", e);
                return false;
            }
        };

        let mut img_rgba = img.to_rgba8();
        let (width, height) = img_rgba.dimensions();

        let redactions: Vec<_> = self
            .redaction_areas
            .iter()
            .filter(|r| &r.file_path == current_file)
            .collect();

        for redaction in redactions {
            let x = (redaction.x * width as f32) as u32;
            let y = (redaction.y * height as f32) as u32;
            let w = (redaction.width * width as f32) as u32;
            let h = (redaction.height * height as f32) as u32;

            for py in y..(y + h).min(height) {
                for px in x..(x + w).min(width) {
                    img_rgba.put_pixel(px, py, image::Rgba([0, 0, 0, 255]));
                }
            }
        }

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
                    self.last_exported_filename = save_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file")
                        .to_string();
                    true
                }
                Err(e) => {
                    println!("Failed to save: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    fn export_redacted_file(&mut self) {
        let current_file = self.files[self.current_file_index].clone();

        let has_redactions = self
            .redaction_areas
            .iter()
            .any(|r| &r.file_path == &current_file);

        if !has_redactions {
            self.show_no_redactions_dialog = true;
            return;
        }

        let extension = current_file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let export_success = if matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp") {
            self.redact_and_export_image(&current_file)
        } else {
            println!("Export not yet implemented for this file type");
            false
        };

        if export_success {
            self.show_export_success_dialog = true;
        }
    }
}
