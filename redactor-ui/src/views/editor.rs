use eframe::egui;
use egui::StrokeKind;
use redactor_core::processors::pdf::SearchResult;
use redactor_core::processors::docs::DocConverter;
use redactor_core::exporters::rasterized::{RasterizedPdfExporter, RedactionBox};
use anyhow::Context;
use log::{info, error};
use std::collections::HashMap;
use std::path::PathBuf;
use std::collections::BTreeMap;

use crate::renders::{pdf_renderer::{self, PdfRenderState}, image_renderer};

pub struct EditorPage {
    files: Vec<PathBuf>,
    current_file_index: usize,
    search_query: String,
    redaction_areas: Vec<RedactionArea>,
    texture_cache: HashMap<(PathBuf, u16), egui::TextureHandle>, // (path, page_index)
    active_search_result: Option<usize>, // Active Search Results index
    scroll_to_active_result: bool,
    // Image viewing state
    zoom: f32,
    pan_offset: egui::Vec2,
    drag_start: Option<egui::Pos2>,
    redaction_mode: bool,
    hovered_redaction: Option<usize>,
    pending_delete: Option<usize>,
    // PDF state
    pdf_state: PdfRenderState,
    search_results: Vec<SearchResult>,
    // Export feedback
    show_no_redactions_dialog: bool,
    show_export_success_dialog: bool,
    last_exported_filename: String,
    return_to_home: bool,
    // DPI selection for PDF export
    show_dpi_dialog: bool,
    selected_dpi: f32,
    pending_export_file: Option<PathBuf>,
    // PDFIUM error handling
    show_pdf_error_dialog: bool,
    pdf_error_message: String,
    is_document_conversion_error: bool,
    // Document converter for .doc/.docx files
    doc_converter: Option<DocConverter>,
    // Track the actual PDF path (for converted documents, this differs from the original file path)
    current_pdf_path: Option<PathBuf>,
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
        let mut editor = Self {
            files,
            current_file_index: 0,
            search_query: String::new(),
            redaction_areas: Vec::new(),
            texture_cache: HashMap::new(),
            active_search_result: None,
            scroll_to_active_result: false,
            zoom: 1.0,
            pan_offset: egui::Vec2::ZERO,
            drag_start: None,
            redaction_mode: false,
            hovered_redaction: None,
            pending_delete: None,
            pdf_state: PdfRenderState::new(),
            search_results: Vec::new(),
            show_no_redactions_dialog: false,
            show_export_success_dialog: false,
            last_exported_filename: String::new(),
            return_to_home: false,
            show_dpi_dialog: false,
            selected_dpi: 150.0,
            pending_export_file: None,
            show_pdf_error_dialog: false,
            pdf_error_message: String::new(),
            is_document_conversion_error: false,
            doc_converter: None,
            current_pdf_path: None,
        };
        editor.load_current_file();
        editor
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

        if self.show_dpi_dialog {
            egui::Window::new("PDF Export Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("Select DPI level for PDF export:");
                    ui.add_space(8.0);
                    
                    ui.label(egui::RichText::new("Default DPI: 150").italics());
                    ui.label("• Higher DPI = Better quality but larger file size");
                    ui.label("• Lower DPI = Smaller file size but reduced quality");
                    
                    ui.add_space(12.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("DPI:");
                        ui.add(
                            egui::Slider::new(&mut self.selected_dpi, 72.0..=600.0)
                                .step_by(1.0)
                                .show_value(true),
                        );
                    });
                    
                    ui.add_space(12.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Export").clicked() {
                            if let Some(file) = self.pending_export_file.take() {
                                if self.redact_and_export_pdf(&file, self.selected_dpi) {
                                    self.show_export_success_dialog = true;
                                }
                            }
                            self.show_dpi_dialog = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_dpi_dialog = false;
                            self.pending_export_file = None;
                        }
                    });
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

        if self.show_pdf_error_dialog {
            egui::Window::new(if self.is_document_conversion_error {
                "Document Conversion Error"
            } else {
                "PDF Error"
            })
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    let title = if self.is_document_conversion_error {
                        "❌ Failed to convert document to PDF:"
                    } else {
                        "❌ An error occurred while rendering the PDF:"
                    };
                    
                    ui.colored_label(egui::Color32::RED, title);
                    ui.add_space(8.0);

                    // Display the error message in a scrollable area
                    egui::ScrollArea::vertical()
                        .id_salt("pdf_error_message_scroll_area")
                        .max_height(150.0)
                        .show(ui, |ui| {
                            ui.label(&self.pdf_error_message);
                        });
                    
                    ui.add_space(12.0);
                    ui.label("This may be caused by:");
                    
                    if self.is_document_conversion_error {
                        ui.label("  • LibreOffice is not installed");
                        ui.label("  • LibreOffice executable not found in system PATH");
                        ui.label("  • Document file is corrupted or unsupported");
                        ui.label("  • Insufficient permissions to read the document");
                    } else {
                        ui.label("  • PDFium library not found or not properly installed");
                        ui.label("  • Corrupted PDF file");
                        ui.label("  • Incompatible PDF format");
                    }
                    
                    ui.add_space(12.0);
                    if ui.button("OK").clicked() {
                        self.show_pdf_error_dialog = false;
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
            self.pdf_state.clear();
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
        self.pdf_state.current_pdf_page = 0;
        self.current_pdf_path = None;
    }

    fn load_current_file(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let current_file = self.files[self.current_file_index].clone();

        // Check if it's a document that needs conversion
        if self.is_document(&current_file) {
            match self.convert_and_load_document(&current_file) {
                Ok(_) => {
                    self.search_results.clear();
                }
                Err(e) => {
                    self.pdf_error_message = format!("Failed to convert document to PDF: {}", e);
                    self.is_document_conversion_error = true;
                    self.show_pdf_error_dialog = true;
                }
            }
        } else if self.is_pdf(&current_file) {
            self.current_pdf_path = None; // No conversion for native PDFs
            if !self.pdf_state.load_file(&current_file) {
                error!("Failed to load PDF file");
                if let Some(error_msg) = &self.pdf_state.pdf_error_message {
                    self.pdf_error_message = error_msg.clone();
                    self.is_document_conversion_error = false;
                    self.show_pdf_error_dialog = true;
                }
            }
            self.search_results.clear();
        }
    }

    fn goto_next_match(&mut self) {
    if self.search_results.is_empty() {
        return;
    }

    let next = match self.active_search_result {
        Some(i) if i + 1 < self.search_results.len() => i + 1,
        _ => 0,
    };

    self.activate_search_result(next);
    }

    fn goto_prev_match(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let prev = match self.active_search_result {
            Some(i) if i > 0 => i - 1,
            _ => self.search_results.len() - 1,
        };

        self.activate_search_result(prev);
    }

    fn activate_search_result(&mut self, idx: usize) {
        let result = match self.search_results.get(idx) {
            Some(r) => r.clone(),  
            None => return,
        };

        self.active_search_result = Some(idx);
        self.pdf_state.current_pdf_page = result.page_index;

        self.center_on_search_result(&result); 
        self.scroll_to_active_result = true;
    }

    fn center_on_search_result(&mut self, result: &SearchResult) {
        if let Some(engine) = &self.pdf_state.pdf_engine {
            if let Ok((page_width, page_height)) =
                engine.get_page_dimensions(result.page_index)
            {
                let rect = result.rect;

                let cx = rect.left().value as f32 + rect.width().value as f32 / 2.0;
                let cy = rect.bottom().value as f32 + rect.height().value as f32 / 2.0;

                let nx = cx / page_width as f32;
                let ny = 1.0 - (cy / page_height as f32);

                // negative pan pulls content into view
                self.pan_offset = egui::vec2(
                    -nx * 500.0 * self.zoom,
                    -ny * 500.0 * self.zoom,
                );
            }
        }
    }


    fn render_search_results_panel(&mut self, ui: &mut egui::Ui) {

        ui.heading("Search Results");
        ui.separator();

        ui.label(format!("Found {} matches", self.search_results.len()));
        ui.add_space(8.0);

        ui.horizontal(|ui| {

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("▶").clicked() {
                self.goto_next_match();
            }
            if ui.button("◀").clicked() {
                self.goto_prev_match();
            }
        });
    });

        egui::ScrollArea::vertical()
            .id_salt("right_search_results_scroll_area")
            .auto_shrink([false, false])
            .show(ui, |ui| {

                let mut grouped: BTreeMap<u16, Vec<usize>> = BTreeMap::new();

                // Building groups 
                for (idx, result) in self.search_results.iter().enumerate() {
                    grouped
                        .entry(result.page_index)
                        .or_default()
                        .push(idx);
                }

                // Rendering the groups
                for (page_index, results) in grouped {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Page {}", page_index + 1))
                                .strong()
                                .size(14.0),
                        );
                        
                        ui.add_space(4.0);

                        for idx in results {
                            let result = &self.search_results[idx];

                            let preview = if result.text.len() > 40 {
                                format!("{}...", &result.text[..40])
                            } else {
                                result.text.clone()
                            };

                            let response = ui
                                .add(
                                    egui::Button::selectable(
                                        self.active_search_result == Some(idx),
                                        format!("\"{}\"", preview),
                                    )
                                    .sense(egui::Sense::click()),
                                );

                            if self.active_search_result == Some(idx) && self.scroll_to_active_result {
                                ui.scroll_to_rect(response.rect, Some(egui::Align::Center));
                            }

                            if response.clicked() {
                                self.activate_search_result(idx);
                            }
                        }

                        ui.add_space(8.0);
                    });

                }
                self.scroll_to_active_result = false;
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
                    &r.file_path == current_file && r.page_index == Some(self.pdf_state.current_pdf_page)
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
                    &r.file_path != current_file || r.page_index != Some(self.pdf_state.current_pdf_page)
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

        if let Some(engine) = &self.pdf_state.pdf_engine {
            match engine.search(&self.search_query) {
                Ok(results) => {
                    self.search_results = results;
                    info!("Found {} search results", self.search_results.len());
                }
                Err(e) => {
                    error!("Search error: {}", e);
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
            "pdf" => {
                if let Some(pdf_result) = pdf_renderer::render_pdf(
                    &mut self.pdf_state,
                    ui,
                    file_path,
                    ctx,
                    &mut self.zoom,
                    &mut self.pan_offset,
                    self.redaction_mode,
                    &mut self.texture_cache,
                ) {
                    // Draw redaction areas and search highlights on top of the PDF
                    self.draw_search_highlights(ui, &pdf_result.image_rect);
                    self.draw_redaction_areas(ui, file_path, &pdf_result.image_rect, &pdf_result.response, ctx);
                    // Handle interactions (panning/redacting)
                    self.handle_interaction(ui, file_path, &pdf_result.image_rect, &pdf_result.response);
                }
            },
            "jpg" | "jpeg" | "png" | "gif" | "webp" => {
                if let Some(image_result) = image_renderer::render_image(
                    ui,
                    ctx,
                    file_path,
                    &mut self.zoom,
                    &mut self.pan_offset,
                    self.redaction_mode,
                    &mut self.texture_cache,
                ) {
                    // Draw redaction areas on top of the image
                    self.draw_redaction_areas(ui, file_path, &image_result.image_rect, &image_result.response, ctx);
                    // Handle interactions (panning/redacting)
                    self.handle_interaction(ui, file_path, &image_result.image_rect, &image_result.response);
                }
            },
            "doc" | "docx" => {
                // Documents are converted to PDF in load_current_file, so render as PDF
                if let Some(pdf_result) = pdf_renderer::render_pdf(
                    &mut self.pdf_state,
                    ui,
                    file_path,
                    ctx,
                    &mut self.zoom,
                    &mut self.pan_offset,
                    self.redaction_mode,
                    &mut self.texture_cache,
                ) {
                    self.draw_search_highlights(ui, &pdf_result.image_rect);
                    self.draw_redaction_areas(ui, file_path, &pdf_result.image_rect, &pdf_result.response, ctx);
                    self.handle_interaction(ui, file_path, &pdf_result.image_rect, &pdf_result.response);
                }
            },
            _ => {
                ui.label("Unsupported file format");
                ui.label(format!("Extension: {}", extension));
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

    fn is_document(&self, file_path: &PathBuf) -> bool {
        file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                let ext = e.to_lowercase();
                ext == "doc" || ext == "docx"
            })
            .unwrap_or(false)
    }

    fn convert_and_load_document(&mut self, file_path: &PathBuf) -> anyhow::Result<()> {
        // Create converter (or reuse existing one for the same file)
        let converter = DocConverter::new().context("Failed to create document converter")?;
        let pdf_path = converter.convert_to_pdf(file_path.to_str().unwrap())
            .context("Failed to convert document to PDF")?;
        
        // Load the converted PDF
        if !self.pdf_state.load_file(&pdf_path) {
            return Err(anyhow::anyhow!("Failed to load converted PDF"));
        }
        
        // Store the converted PDF path for later export
        self.current_pdf_path = Some(pdf_path);
        
        // Store converter to keep temp directory alive
        self.doc_converter = Some(converter);
        
        Ok(())
    }

    fn draw_search_highlights(&self, ui: &mut egui::Ui, image_rect: &egui::Rect) {
        if let Some(engine) = &self.pdf_state.pdf_engine {
            for (idx, result) in self.search_results.iter().enumerate() {
                if result.page_index == self.pdf_state.current_pdf_page {
                    let rect_bounds = result.rect;

                    if let Ok((page_width, page_height)) =
                        engine.get_page_dimensions(self.pdf_state.current_pdf_page)
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
            Some(self.pdf_state.current_pdf_page)
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
                    Some(self.pdf_state.current_pdf_page)
                } else {
                    None
                },
            });
        }
    }

    fn redact_and_export_pdf(&mut self, current_file: &PathBuf, dpi: f32) -> bool {
        let redactions: Vec<RedactionBox> = self
            .redaction_areas
            .iter()
            .filter(|r| &r.file_path == current_file)
            .map(|r| RedactionBox {
                page_index: r.page_index.unwrap_or(0),
                x: r.x,
                y: r.y,
                width: r.width,
                height: r.height,
            })
            .collect();

        // Use the converted PDF path if available (for .doc/.docx files), otherwise use the original path
        let pdf_source_path = self.current_pdf_path.as_ref().unwrap_or(current_file);

        if let Some(save_path) = rfd::FileDialog::new()
            .set_file_name(format!(
                "{}_redacted.pdf",
                current_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("document")
            ))
            .add_filter("PDF Document", &["pdf"])
            .save_file()
        {
            match RasterizedPdfExporter::export_with_redactions(
                &pdf_source_path.to_string_lossy(),
                &save_path.to_string_lossy(),
                redactions,
                dpi,
            ) {
                Ok(_) => {
                    self.last_exported_filename = save_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file")
                        .to_string();
                    true
                }
                Err(e) => {
                    error!("Failed to export PDF: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }


    fn redact_and_export_image(&mut self, current_file: &PathBuf) -> bool {
        let img = match image::open(current_file) {
            Ok(img) => img,
            Err(e) => {
                error!("Failed to load image for export: {}", e);
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
                    error!("Failed to save: {}", e);
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

        // For PDFs, show DPI selection dialog
        if matches!(extension.as_str(), "pdf" | "doc" | "docx") {
            self.pending_export_file = Some(current_file);
            self.show_dpi_dialog = true;
            return;
        }

        // For images, export directly
        let export_success = if matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp") {
            self.redact_and_export_image(&current_file)
        } else {
            error!("Export not yet implemented for this file type");
            false
        };

        if export_success {
            self.show_export_success_dialog = true;
        }
    }
}
