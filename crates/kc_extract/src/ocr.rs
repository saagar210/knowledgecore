use kc_core::app_error::{AppError, AppResult};

pub struct OcrConfig {
    pub tesseract_cmd: Option<String>,
    pub language: String,
}

pub fn should_run_ocr(extracted_len: usize, alnum_ratio: f64) -> bool {
    extracted_len < 800 || alnum_ratio < 0.10
}

pub fn ocr_pdf_via_images(_pdf_bytes: &[u8], _ocr_cfg: &OcrConfig) -> AppResult<String> {
    Err(AppError::new(
        "KC_TESSERACT_UNAVAILABLE",
        "extract",
        "ocr pipeline is unavailable in this environment",
        true,
        serde_json::json!({}),
    ))
}
