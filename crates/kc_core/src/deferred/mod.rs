use crate::app_error::AppError;
use serde::{Deserialize, Serialize};

pub mod encryption;
pub mod lineage;
pub mod sync;
pub mod zip_packaging;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DraftCapabilityStatusV1 {
    pub schema_version: i64,
    pub status: String,
    pub capability: String,
    pub activation_phase: String,
    pub spec_path: String,
    pub preview_error_code: String,
}

pub fn preview_capability_statuses() -> Vec<DraftCapabilityStatusV1> {
    vec![
        DraftCapabilityStatusV1 {
            schema_version: 1,
            status: "draft".to_string(),
            capability: "encryption".to_string(),
            activation_phase: "M".to_string(),
            spec_path: "spec/22-encryption-at-rest-v1-design-lock.md".to_string(),
            preview_error_code: encryption::PREVIEW_ERROR_CODE.to_string(),
        },
        DraftCapabilityStatusV1 {
            schema_version: 1,
            status: "draft".to_string(),
            capability: "lineage".to_string(),
            activation_phase: "N3".to_string(),
            spec_path: "spec/25-advanced-lineage-ui-v1-design-lock.md".to_string(),
            preview_error_code: lineage::PREVIEW_ERROR_CODE.to_string(),
        },
        DraftCapabilityStatusV1 {
            schema_version: 1,
            status: "draft".to_string(),
            capability: "sync".to_string(),
            activation_phase: "N2".to_string(),
            spec_path: "spec/24-cross-device-sync-v1-design-lock.md".to_string(),
            preview_error_code: sync::PREVIEW_ERROR_CODE.to_string(),
        },
        DraftCapabilityStatusV1 {
            schema_version: 1,
            status: "draft".to_string(),
            capability: "zip_packaging".to_string(),
            activation_phase: "N1".to_string(),
            spec_path: "spec/23-deterministic-zip-packaging-v1-design-lock.md".to_string(),
            preview_error_code: zip_packaging::PREVIEW_ERROR_CODE.to_string(),
        },
    ]
}

pub fn scaffold_error_for_capability(capability: &str) -> AppError {
    match capability {
        "encryption" => encryption::not_implemented_error(),
        "zip_packaging" => zip_packaging::not_implemented_error(),
        "sync" => sync::not_implemented_error(),
        "lineage" => lineage::not_implemented_error(),
        _ => AppError::new(
            "KC_DRAFT_PREVIEW_UNKNOWN_CAPABILITY",
            "draft",
            "unknown draft preview capability",
            false,
            serde_json::json!({
                "capability": capability,
                "supported": ["encryption", "lineage", "sync", "zip_packaging"]
            }),
        ),
    }
}
