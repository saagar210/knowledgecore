use crate::app_error::AppError;
use serde::{Deserialize, Serialize};

pub const PREVIEW_ERROR_CODE: &str = "KC_DRAFT_ENCRYPTION_NOT_IMPLEMENTED";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KdfParamsDraftV1 {
    pub algorithm: String,
    pub memory_kib: i64,
    pub iterations: i64,
    pub parallelism: i64,
    pub salt_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptionMetadataDraftV1 {
    pub schema_version: i64,
    pub status: String,
    pub activation_phase: String,
    pub cipher_suite: String,
    pub kdf: KdfParamsDraftV1,
    pub key_reference: String,
}

pub fn placeholder_metadata() -> EncryptionMetadataDraftV1 {
    EncryptionMetadataDraftV1 {
        schema_version: 1,
        status: "draft".to_string(),
        activation_phase: "M".to_string(),
        cipher_suite: "xchacha20poly1305".to_string(),
        kdf: KdfParamsDraftV1 {
            algorithm: "argon2id".to_string(),
            memory_kib: 65536,
            iterations: 3,
            parallelism: 1,
            salt_id: "draft-salt-id-v1".to_string(),
        },
        key_reference: "draft-key-reference-v1".to_string(),
    }
}

pub fn not_implemented_error() -> AppError {
    AppError::new(
        PREVIEW_ERROR_CODE,
        "draft",
        "encryption capability is not implemented in Phase L",
        false,
        serde_json::json!({
            "activation_phase": "M",
            "schema_status": "draft",
            "spec": "spec/22-encryption-at-rest-v1-design-lock.md"
        }),
    )
}
