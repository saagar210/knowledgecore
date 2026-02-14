use crate::app_error::{AppError, AppResult};

pub fn blake3_hex_prefixed(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

pub fn validate_blake3_prefixed(s: &str) -> AppResult<()> {
    if !s.starts_with("blake3:") {
        return Err(AppError::new(
            "KC_HASH_INVALID_FORMAT",
            "hash",
            "hash must start with blake3:",
            false,
            serde_json::json!({ "value": s }),
        ));
    }
    let hex = &s[7..];
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()) {
        return Err(AppError::new(
            "KC_HASH_DECODE_FAILED",
            "hash",
            "invalid lowercase hex digest",
            false,
            serde_json::json!({ "value": s }),
        ));
    }
    Ok(())
}
