#[test]
fn extract_smoke() {
    let _ = kc_extract::DefaultExtractor::new(kc_core::services::ToolchainIdentity {
        pdfium_identity: "pdfium-test".to_string(),
        tesseract_identity: "tesseract-test".to_string(),
    });
}
