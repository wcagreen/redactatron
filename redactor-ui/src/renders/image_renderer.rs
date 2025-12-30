use eframe::egui;
use std::path::PathBuf;

pub struct ImageRenderResult {
    pub image_rect: egui::Rect,
    pub response: egui::Response,
}

pub fn render_image(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    file_path: &PathBuf,
    zoom: &mut f32,
    pan_offset: &mut egui::Vec2,
    redaction_mode: bool,
    texture_cache: &mut std::collections::HashMap<(PathBuf, u16), egui::TextureHandle>,
) -> Option<ImageRenderResult> {
    let cache_key = (file_path.clone(), 0);

    if !texture_cache.contains_key(&cache_key) {
        match image::open(file_path) {
            Ok(img) => {
                let img_rgba = img.to_rgba8();
                let size = [img_rgba.width() as _, img_rgba.height() as _];
                let pixels = img_rgba.as_flat_samples();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                let texture =
                    ctx.load_texture(file_path.to_string_lossy(), color_image, Default::default());

                texture_cache.insert(cache_key.clone(), texture);
            }
            Err(e) => {
                ui.colored_label(egui::Color32::RED, format!("Failed to load image: {}", e));
                return None;
            }
        }
    }

    if let Some(texture) = texture_cache.get(&cache_key) {
        let original_size = texture.size_vec2();
        let texture_id = texture.id();
        let scaled_size = original_size * *zoom;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Zoom:");
                if ui.button("➖").clicked() {
                    *zoom = (*zoom - 0.1).max(0.1);
                }
                ui.label(format!("{:.0}%", *zoom * 100.0));
                if ui.button("➕").clicked() {
                    *zoom = (*zoom + 0.1).min(5.0);
                }
                if ui.button("Reset View").clicked() {
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
                    ui.label("🖐️ PAN MODE - Drag to move image");
                }
            });

            ui.separator();

            let (rect, response) =
                ui.allocate_exact_size(scaled_size, egui::Sense::click_and_drag());

            if response.hovered() {
                let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll_delta != 0.0 {
                    *zoom = (*zoom + scroll_delta * 0.001).clamp(0.1, 5.0);
                }
            }

            let image_rect = egui::Rect::from_min_size(rect.min + *pan_offset, scaled_size);

            ui.painter().image(
                texture_id,
                image_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            ui.add_space(10.0);
            ui.label(format!(
                "Original size: {}x{} pixels",
                original_size.x, original_size.y
            ));

            Some(ImageRenderResult {
                image_rect,
                response: response.clone(),
            })
        })
        .inner
    } else {
        None
    }
}
