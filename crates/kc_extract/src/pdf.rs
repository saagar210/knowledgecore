use crate::markers::page_marker;
use kc_core::app_error::{AppError, AppResult};
use std::fs;
use std::process::Command;

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

fn parse_pages(text: &str) -> String {
    let mut output = String::new();
    for (idx, page) in text.split('\u{000C}').enumerate() {
        let trimmed = page.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&page_marker(idx + 1));
        output.push('\n');
        output.push_str(trimmed);
    }
    if output.is_empty() {
        output = format!("{}\n", page_marker(1));
    }
    output
}

fn extract_pdf_via_command(pdf_bytes: &[u8]) -> AppResult<String> {
    let dir = tempfile::tempdir().map_err(|e| {
        AppError::new(
            "KC_CANONICAL_EXTRACT_FAILED",
            "extract",
            "failed creating temporary directory for pdf extraction",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    let input_path = dir.path().join("input.pdf");
    fs::write(&input_path, pdf_bytes).map_err(|e| {
        AppError::new(
            "KC_CANONICAL_EXTRACT_FAILED",
            "extract",
            "failed writing temporary pdf file",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;

    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg("-nopgbrk")
        .arg(&input_path)
        .arg("-")
        .output()
        .map_err(|e| {
            AppError::new(
                "KC_PDFIUM_UNAVAILABLE",
                "extract",
                "pdftotext/PDF backend is unavailable",
                true,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;

    if !output.status.success() {
        return Err(AppError::new(
            "KC_CANONICAL_EXTRACT_FAILED",
            "extract",
            "pdf text extraction command failed",
            false,
            serde_json::json!({
                "status": output.status.code(),
                "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
            }),
        ));
    }

    let text = String::from_utf8(output.stdout).map_err(|e| {
        AppError::new(
            "KC_CANONICAL_EXTRACT_FAILED",
            "extract",
            "pdf extraction output was not utf8",
            false,
            serde_json::json!({ "error": e.to_string() }),
        )
    })?;
    Ok(parse_pages(&text))
}

pub fn extract_pdf_text(pdf_bytes: &[u8], _cfg: &PdfiumConfig) -> AppResult<PdfExtractOutput> {
    let text = if pdf_bytes.starts_with(b"%PDF") {
        extract_pdf_via_command(pdf_bytes)?
    } else {
        let decoded = String::from_utf8(pdf_bytes.to_vec()).map_err(|e| {
            AppError::new(
                "KC_CANONICAL_EXTRACT_FAILED",
                "extract",
                "pdf bytes could not be decoded as utf8",
                false,
                serde_json::json!({ "error": e.to_string() }),
            )
        })?;
        format!("{}\n{}", page_marker(1), decoded)
    };
    let ratio = alnum_ratio(&text);

    Ok(PdfExtractOutput {
        extracted_len: text.len(),
        extracted_alnum_ratio: ratio,
        text_with_page_markers: text,
    })
}
