use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};
use lopdf::Document;
use log::{debug, info};

use crate::processors::pdf::PdfEngine;

pub struct RasterizedPdfExporter;

#[derive(Clone, Debug)]
pub struct RedactionBox {
    pub page_index: u16,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RasterizedPdfExporter {
    /// Export PDF with redactions as a rasterized PDF
    ///
    /// # Arguments
    /// * `input_path` - Path to source PDF
    /// * `output_path` - Path to save redacted PDF
    /// * `redactions` - Vector of redaction boxes (normalized 0.0-1.0 coordinates)
    /// * `dpi` - DPI for rendering (e.g., 150 for good quality, 300 for high quality)
    pub fn export_with_redactions(
        input_path: &str,
        output_path: &str,
        redactions: Vec<RedactionBox>,
        dpi: f32,
    ) -> Result<()> {
        info!("Starting rasterized PDF export: input={}, output={}, dpi={}, redaction_count={}", input_path, output_path, dpi, redactions.len());
        let mut engine = PdfEngine::new();
        engine.load_file(input_path)?;

        let page_count = engine.page_count();
        let scale_factor = dpi / 72.0; // Convert DPI to scale factor (72 DPI is standard)
        debug!("Scale factor calculated: {}", scale_factor);

        let mut rasterized_pages = Vec::new();

        // Render all pages with redactions applied
        for page_idx in 0..page_count {
            debug!("Processing page {} of {}", page_idx + 1, page_count);
            let mut rendered_img = engine.render_page(page_idx, scale_factor)?;

            // Apply redactions to this page
            Self::apply_redactions_to_image(&mut rendered_img, page_idx, &redactions)?;

            rasterized_pages.push(rendered_img);
        }

        debug!("All {} pages rendered and redacted", page_count);
        // Create PDF from rasterized images
        Self::create_pdf_from_images(&rasterized_pages, output_path)?;

        info!("Rasterized PDF export completed successfully: {}", output_path);
        Ok(())
    }

    /// Apply redaction boxes to a rendered image
    fn apply_redactions_to_image(
        img: &mut DynamicImage,
        page_index: u16,
        redactions: &[RedactionBox],
    ) -> Result<()> {
        let (width, height) = img.dimensions();

        // Filter redactions for this page
        let page_redactions: Vec<_> = redactions
            .iter()
            .filter(|r| r.page_index == page_index)
            .collect();

        if page_redactions.is_empty() {
            debug!("No redactions to apply for page {}", page_index);
            return Ok(());
        }

        debug!("Applying {} redactions to page {}", page_redactions.len(), page_index);
        let mut img_rgba = img.to_rgba8();

        for redaction in page_redactions {
            // Convert normalized coordinates to pixel coordinates
            let x = (redaction.x * width as f32) as u32;
            let y = (redaction.y * height as f32) as u32;
            let w = (redaction.width * width as f32) as u32;
            let h = (redaction.height * height as f32) as u32;

            debug!("Redacting area at ({},{}) size {}x{} on page {}", x, y, w, h, page_index);
            // Draw black rectangle over redacted area
            for py in y..(y + h).min(height) {
                for px in x..(x + w).min(width) {
                    img_rgba.put_pixel(px, py, image::Rgba([0, 0, 0, 255]));
                }
            }
        }

        *img = DynamicImage::ImageRgba8(img_rgba);
        Ok(())
    }

    /// Create a PDF from a vector of images using lopdf
    fn create_pdf_from_images(images: &[DynamicImage], output_path: &str) -> Result<()> {
        debug!("Creating PDF from {} rasterized images", images.len());
        let mut doc = Document::new();

        let mut page_ids = vec![];

        for (idx, img) in images.iter().enumerate() {
            debug!("Processing image {}/{}", idx + 1, images.len());
            let img_rgb = img.to_rgb8();
            let (width, height) = img.dimensions();
            debug!("Image dimensions: {}x{}", width, height);

            // Create image stream
            let image_stream = lopdf::Stream::new(Default::default(), img_rgb.to_vec());
            let mut image_dict = image_stream.dict.clone();
            image_dict.set("Type", "XObject");
            image_dict.set("Subtype", "Image");
            image_dict.set("Width", width as i32);
            image_dict.set("Height", height as i32);
            image_dict.set("ColorSpace", "DeviceRGB");
            image_dict.set("BitsPerComponent", 8);

            let image_stream = lopdf::Stream::new(image_dict, img_rgb.to_vec());
            let image_id = doc.add_object(image_stream);

            // Create content stream for page
            let content = format!(
                "q\n{} 0 0 {} 0 0 cm\n/Image{} Do\nQ\n",
                width, height, idx
            );
            let content_stream = lopdf::Stream::new(Default::default(), content.into_bytes());
            let content_id = doc.add_object(content_stream);

            // Create page dictionary
            let mut page_dict = lopdf::Dictionary::new();
            page_dict.set("Type", "Page");
            page_dict.set(
                "MediaBox",
                vec![0.into(), 0.into(), (width as i32).into(), (height as i32).into()],
            );
            page_dict.set("Contents", content_id);

            // Set up resources with image reference
            let mut resources = lopdf::Dictionary::new();
            let mut xobjects = lopdf::Dictionary::new();
            xobjects.set(format!("Image{}", idx), image_id);
            resources.set("XObject", xobjects);
            page_dict.set("Resources", resources);

            let page_id = doc.add_object(lopdf::Object::Dictionary(page_dict));
            page_ids.push(page_id);
        }

        // Create pages tree
        let mut pages_dict = lopdf::Dictionary::new();
        pages_dict.set("Type", "Pages");
        
        // Convert ObjectIds to Objects for the Kids array
        let kids: Vec<lopdf::Object> = page_ids.into_iter().map(|id| id.into()).collect();
        pages_dict.set("Kids", kids);
        pages_dict.set("Count", images.len() as i32);

        let pages_id = doc.add_object(lopdf::Object::Dictionary(pages_dict));

        // Create catalog
        let mut catalog_dict = lopdf::Dictionary::new();
        catalog_dict.set("Type", "Catalog");
        catalog_dict.set("Pages", pages_id);

        let catalog_id = doc.add_object(lopdf::Object::Dictionary(catalog_dict));
        doc.trailer.set("Root", catalog_id);

        // Save document
        debug!("Saving PDF document to: {}", output_path);
        doc.save(output_path)
            .context("Failed to save PDF document")?;
        info!("PDF document successfully saved with {} pages", images.len());

        Ok(())
    }
}
