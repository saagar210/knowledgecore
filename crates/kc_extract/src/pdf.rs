use crate::markers::page_marker;
use kc_core::app_error::{AppError, AppResult};

pub struct PdfiumConfig {
    pub library_path: Option<String>,
}

pub struct PdfExtractOutput {
    pub text_with_page_markers: String,
    pub extracted_len: usize,
    pub extracted_alnum_ratio: f64,
}

fn alnum_ratio(content: &str) -> f64 {
    let mut total = 0usize;
    let mut alnum = 0usize;
    for ch in content.chars() {
        if ch == '[' || ch == ']' {
            continue;
        }
        total += 1;
        if ch.is_ascii_alphanumeric() {
            alnum += 1;
        }
    }
    if total == 0 {
        return 0.0;
    }
    alnum as f64 / total as f64
}

pub fn extract_pdf_text(pdf_bytes: &[u8], _cfg: &PdfiumConfig) -> AppResult<PdfExtractOutput> {
    let decoded = String::from_utf8(pdf_bytes.to_vec()).map_err(|e| {
        AppError::new(
            "KC_CANONICAL_EXTRACT_FAILED",
            "extract",
            "pdf bytes could not be decoded as utf8 in stub extractor",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    let text = format!("{}\n{}", page_marker(1), decoded);
    let ratio = alnum_ratio(&text);

    Ok(PdfExtractOutput {
        extracted_len: text.len(),
        extracted_alnum_ratio: ratio,
        text_with_page_markers: text,
    })
}
