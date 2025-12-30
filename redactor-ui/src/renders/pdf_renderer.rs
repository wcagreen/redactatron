use eframe::egui;
use redactor_core::processors::pdf::PdfEngine;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct PdfRenderState {
    pub pdf_engine: Option<PdfEngine>,
    pub current_pdf_page: u16,
    pub pdf_page_count: u16,
    pub pdf_error_message: Option<String>,
}

impl PdfRenderState {
    pub fn new() -> Self {
        Self {
            pdf_engine: None,
            current_pdf_page: 0,
            pdf_page_count: 0,
            pdf_error_message: None,
        }
    }

    pub fn load_file(&mut self, file_path: &PathBuf) -> bool {
        let mut engine = PdfEngine::new();
        match engine.load_file(&file_path.to_string_lossy()) {
            Ok(_) => {
                self.pdf_page_count = engine.page_count();
                self.current_pdf_page = 0;
                self.pdf_engine = Some(engine);
                self.pdf_error_message = None;
                true
            }
            Err(e) => {
                self.pdf_error_message = Some(e.to_string());
                false
            }
        }
    }

    pub fn clear(&mut self) {
        self.pdf_engine = None;
        self.current_pdf_page = 0;
        self.pdf_page_count = 0;
    }
}

pub struct PdfRenderResult {
    pub image_rect: egui::Rect,
    pub response: egui::Response,
}

pub fn render_pdf(
    pdf_state: &mut PdfRenderState,
    ui: &mut egui::Ui,
    file_path: &PathBuf,
    ctx: &egui::Context,
    zoom: &mut f32,
    pan_offset: &mut egui::Vec2,
    redaction_mode: bool,
    texture_cache: &mut HashMap<(PathBuf, u16), egui::TextureHandle>,
) -> Option<PdfRenderResult> {
    // Initialize PDF engine if needed
    if pdf_state.pdf_engine.is_none() {
        if !pdf_state.load_file(file_path) {
            ui.colored_label(egui::Color32::RED, "Failed to load PDF");
            return None;
        }
    }

    let page_count = pdf_state.pdf_page_count;

    ui.vertical(|ui| {
        // Page navigation and controls
        ui.horizontal(|ui| {
            ui.label("Page:");
            if ui.button("◀").clicked() && pdf_state.current_pdf_page > 0 {
                pdf_state.current_pdf_page -= 1;
                *pan_offset = egui::Vec2::ZERO;
            }

            ui.label(format!(
                "{} / {}",
                pdf_state.current_pdf_page + 1,
                page_count
            ));

            if ui.button("▶").clicked() && pdf_state.current_pdf_page < page_count - 1 {
                pdf_state.current_pdf_page += 1;
                *pan_offset = egui::Vec2::ZERO;
            }

            ui.separator();

            // Zoom controls
            ui.label("Zoom:");
            if ui.button("➖").clicked() {
                *zoom = (*zoom - 0.1).max(0.5);
            }
            ui.label(format!("{:.0}%", *zoom * 100.0));
            if ui.button("➕").clicked() {
                *zoom = (*zoom + 0.1).min(3.0);
            }
            if ui.button("Reset").clicked() {
                *zoom = 1.0;
                *pan_offset = egui::Vec2::ZERO;
            }

            ui.separator();

            if redaction_mode {
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
        let cache_key = (file_path.clone(), pdf_state.current_pdf_page);

        if !texture_cache.contains_key(&cache_key) {
            if let Some(engine) = &pdf_state.pdf_engine {
                match engine.render_page(pdf_state.current_pdf_page, 2.0) {
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
                                pdf_state.current_pdf_page
                            ),
                            color_image,
                            Default::default(),
                        );

                        texture_cache.insert(cache_key.clone(), texture);
                    }
                    Err(e) => {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("Failed to render page: {}", e),
                        );
                        return None;
                    }
                }
            }
        }

        if let Some(texture) = texture_cache.get(&cache_key) {
            let texture_id = texture.id();
            let texture_size = texture.size_vec2();

            render_pdf_page_with_highlights(ui, ctx, texture_id, texture_size, zoom, pan_offset)
        } else {
            None
        }
    })
    .inner
}

pub fn render_pdf_page_with_highlights(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    texture_id: egui::TextureId,
    texture_size: egui::Vec2,
    zoom: &mut f32,
    pan_offset: &mut egui::Vec2,
) -> Option<PdfRenderResult> {
    let scaled_size = texture_size * *zoom;

    let (rect, response) = ui.allocate_exact_size(scaled_size, egui::Sense::click_and_drag());

    // Handle zoom with scroll wheel
    if response.hovered() {
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            *zoom = (*zoom + scroll_delta * 0.001).clamp(0.5, 3.0);
        }
    }

    let image_rect = egui::Rect::from_min_size(rect.min + *pan_offset, scaled_size);

    // Draw the PDF page
    ui.painter().image(
        texture_id,
        image_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    Some(PdfRenderResult {
        image_rect,
        response: response.clone(),
    })
}
