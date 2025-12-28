use anyhow::{Context, Result};
use image::DynamicImage;
use pdfium_render::prelude::{PdfRect, PdfSearchDirection, PdfSearchOptions, Pdfium};
use std::cell::RefCell;

// Thread-local Pdfium instance for PDF processing. This was about annoying to set up, likely better way to do it.
thread_local! {
    static PDFIUM: RefCell<Option<Pdfium>> = RefCell::new(None);
}

pub struct PdfEngine {
    pdf_data: Vec<u8>,
    page_count: u16,
}

impl PdfEngine {
    pub fn new() -> Self {
        Self {
            pdf_data: Vec::new(),
            page_count: 0,
        }
    }

    /// Initializes Pdfium for the current thread if not already done. Likely not needed better way to do this.
    fn ensure_pdfium<F, R>(f: F) -> Result<R>
    where
        F: FnOnce(&Pdfium) -> Result<R>,
    {
        PDFIUM.with(|cell| {
            let mut opt = cell.borrow_mut();
            if opt.is_none() {
                let bindings =
                    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                        .or_else(|_| Pdfium::bind_to_system_library())
                        .map_err(|e| anyhow::anyhow!("Failed to bind PDFium: {}", e))?;
                *opt = Some(Pdfium::new(bindings));
            }
            f(opt.as_ref().unwrap())
        })
    }

    pub fn load_file(&mut self, path: &str) -> Result<()> {
        // Storing data in engine to make it portable across threads if needed Likley better way to load the pdf it was bit annoying to figure this out.
        self.pdf_data = std::fs::read(path).context("Failed to read PDF file")?;

        Self::ensure_pdfium(|pdfium| {
            let doc = pdfium.load_pdf_from_byte_vec(self.pdf_data.clone(), None)?;
            self.page_count = doc.pages().len();
            Ok(())
        })?;

        Ok(())
    }

    pub fn page_count(&self) -> u16 {
        self.page_count
    }

    pub fn render_page(&self, page_index: u16, scale_factor: f32) -> Result<DynamicImage> {
        Self::ensure_pdfium(|pdfium| {
            let doc = pdfium.load_pdf_from_byte_vec(self.pdf_data.clone(), None)?;
            let page = doc.pages().get(page_index)?;

            let width = (page.width().value * scale_factor) as i32;
            let height = (page.height().value * scale_factor) as i32;

            Ok(page.render(width, height, None)?.as_image())
        })
    }

    pub fn search(&self, term: &str) -> Result<Vec<SearchResult>> {
        if self.pdf_data.is_empty() {
            return Ok(vec![]);
        }

        Self::ensure_pdfium(|pdfium| {
            let doc = pdfium.load_pdf_from_byte_vec(self.pdf_data.clone(), None)?;
            let mut results = Vec::new();

            let options = PdfSearchOptions::new()
                .match_case(false)
                .match_whole_word(false);

            for (page_idx, page) in doc.pages().iter().enumerate() {
                let text = page.text()?;
                let search = text.search(term, &options)?;

                for segments in search.iter(PdfSearchDirection::SearchForward) {
                    for segment in segments.iter() {
                        let bounds = segment.bounds();
                        results.push(SearchResult {
                            page_index: page_idx as u16,
                            rect: bounds,
                            text: segment.text().to_string(),
                        });
                    }
                }
            }

            Ok(results)
        })
    }

    pub fn get_page_dimensions(&self, page_index: u16) -> Result<(f32, f32)> {
        Self::ensure_pdfium(|pdfium| {
            let doc = pdfium.load_pdf_from_byte_vec(self.pdf_data.clone(), None)?;
            let page = doc.pages().get(page_index)?;

            Ok((page.width().value, page.height().value))
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub page_index: u16,
    pub rect: PdfRect,
    pub text: String,
}
