use crate::app_error::AppError;
use serde::{Deserialize, Serialize};

pub const PREVIEW_ERROR_CODE: &str = "KC_DRAFT_ZIP_PACKAGING_NOT_IMPLEMENTED";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZipPackagingMetadataDraftV1 {
    pub schema_version: i64,
    pub status: String,
    pub activation_phase: String,
    pub format: String,
    pub entry_order: String,
    pub timestamp_policy: String,
    pub permission_policy: String,
}

pub fn placeholder_metadata() -> ZipPackagingMetadataDraftV1 {
    ZipPackagingMetadataDraftV1 {
        schema_version: 1,
        status: "draft".to_string(),
        activation_phase: "N1".to_string(),
        format: "zip".to_string(),
        entry_order: "lexicographic_path".to_string(),
        timestamp_policy: "fixed_epoch_ms".to_string(),
        permission_policy: "normalized_posix_mode".to_string(),
    }
}

pub fn not_implemented_error() -> AppError {
    AppError::new(
        PREVIEW_ERROR_CODE,
        "draft",
        "deterministic zip packaging is not implemented in Phase L",
        false,
        serde_json::json!({
            "activation_phase": "N1",
            "schema_status": "draft",
            "spec": "spec/23-deterministic-zip-packaging-v1-design-lock.md"
        }),
    )
}
