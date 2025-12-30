use redactor_core::processors::pdf::{PdfEngine, SearchResult};
use pdfium_render::prelude::{PdfRect, PdfPoints};

#[test]
fn test_pdf_engine_creation() {
    let engine = PdfEngine::new();
    assert_eq!(engine.page_count(), 0);
}


#[test]
fn test_search_result_creation() {
    let search_result = SearchResult {
        page_index: 1,
        rect: PdfRect::new(
            PdfPoints::new(10.0),
            PdfPoints::new(20.0),
            PdfPoints::new(30.0),
            PdfPoints::new(40.0),
        ),
        text: "test".to_string(),
    };
    assert_eq!(search_result.page_index, 1);
    assert_eq!(search_result.text, "test");
}

