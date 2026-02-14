use crate::app_error::AppError;
use serde::{Deserialize, Serialize};

pub const PREVIEW_ERROR_CODE: &str = "KC_DRAFT_SYNC_NOT_IMPLEMENTED";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncManifestDraftV1 {
    pub schema_version: i64,
    pub status: String,
    pub activation_phase: String,
    pub vault_id: String,
    pub snapshot_id: String,
    pub created_at_ms: i64,
    pub objects_hash: String,
    pub db_hash: String,
    pub conflicts: Vec<SyncConflictDraftV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncConflictDraftV1 {
    pub path: String,
    pub local_hash: String,
    pub remote_hash: String,
    pub resolution_strategy: String,
}

pub fn placeholder_manifest() -> SyncManifestDraftV1 {
    SyncManifestDraftV1 {
        schema_version: 1,
        status: "draft".to_string(),
        activation_phase: "N2".to_string(),
        vault_id: "draft-vault-id".to_string(),
        snapshot_id: "draft-snapshot-id".to_string(),
        created_at_ms: 0,
        objects_hash: "blake3:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        db_hash: "blake3:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        conflicts: vec![SyncConflictDraftV1 {
            path: "docs/note.md".to_string(),
            local_hash: "blake3:1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            remote_hash: "blake3:2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            resolution_strategy: "emit_conflict_artifact".to_string(),
        }],
    }
}

pub fn not_implemented_error() -> AppError {
    AppError::new(
        PREVIEW_ERROR_CODE,
        "draft",
        "cross-device sync is not implemented in Phase L",
        false,
        serde_json::json!({
            "activation_phase": "N2",
            "schema_status": "draft",
            "spec": "spec/24-cross-device-sync-v1-design-lock.md"
        }),
    )
}
