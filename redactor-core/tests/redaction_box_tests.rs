use redactor_core::exporters::rasterized::RedactionBox;

#[test]
fn test_redaction_box_creation() {
    let box_rect = RedactionBox {
        page_index: 0,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
    };
    assert_eq!(box_rect.page_index, 0);
    assert_eq!(box_rect.x, 10.0);
    assert_eq!(box_rect.y, 20.0);
    assert_eq!(box_rect.width, 100.0);
    assert_eq!(box_rect.height, 50.0);
}

#[test]
fn test_redaction_box_normalized_coordinates() {
    let box_rect = RedactionBox {
        page_index: 0,
        x: 0.5,
        y: 0.5,
        width: 0.25,
        height: 0.25,
    };
    assert!(box_rect.x >= 0.0 && box_rect.x <= 1.0);
    assert!(box_rect.y >= 0.0 && box_rect.y <= 1.0);
    assert!(box_rect.width >= 0.0 && box_rect.width <= 1.0);
    assert!(box_rect.height >= 0.0 && box_rect.height <= 1.0);
}

#[test]
fn test_redaction_box_different_pages() {
    let box1 = RedactionBox {
        page_index: 0,
        x: 0.1,
        y: 0.1,
        width: 0.2,
        height: 0.2,
    };
    let box2 = RedactionBox {
        page_index: 1,
        x: 0.1,
        y: 0.1,
        width: 0.2,
        height: 0.2,
    };
    assert_ne!(box1.page_index, box2.page_index);
}
