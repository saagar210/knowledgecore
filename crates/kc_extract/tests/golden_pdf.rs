use kc_core::services::{ExtractInput, ExtractService, ToolchainIdentity};
use kc_core::types::DocId;
use kc_extract::ocr::should_run_ocr;
use kc_extract::pdf::{extract_pdf_text, PdfiumConfig};
use kc_extract::DefaultExtractor;

#[test]
fn golden_pdf_adds_page_marker() {
    let pdf = extract_pdf_text(b"hello page", &PdfiumConfig { library_path: None }).expect("extract");
    assert!(pdf.text_with_page_markers.starts_with("[[PAGE:0001]]"));
}

#[test]
fn golden_pdf_ocr_trigger_threshold() {
    assert!(should_run_ocr(100, 0.5));
    assert!(should_run_ocr(1000, 0.05));
    assert!(!should_run_ocr(1000, 0.5));
}

#[test]
fn golden_pdf_extractor_hashes_canonical() {
    let extractor = DefaultExtractor::new(ToolchainIdentity {
        pdfium_identity: "pdfium:test".to_string(),
        tesseract_identity: "tesseract:test".to_string(),
    });

    let out = extractor
        .extract_canonical(ExtractInput {
            doc_id: &DocId("blake3:3333333333333333333333333333333333333333333333333333333333333333".to_string()),
            bytes: b"pdf body",
            mime: "application/pdf",
            source_kind: "manuals",
        })
        .expect("extract");

    assert!(out.canonical_hash.0.starts_with("blake3:"));
}
