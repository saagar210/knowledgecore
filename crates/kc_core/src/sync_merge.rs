use crate::app_error::{AppError, AppResult};
use crate::hashing::validate_blake3_prefixed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SyncMergeChangeSetV1 {
    #[serde(default)]
    pub object_hashes: Vec<String>,
    #[serde(default)]
    pub lineage_overlay_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncMergePreviewReportV1 {
    pub schema_version: i64,
    pub merge_policy: String,
    pub safe: bool,
    pub generated_at_ms: i64,
    pub local: SyncMergeChangeSetV1,
    pub remote: SyncMergeChangeSetV1,
    pub overlap: SyncMergeChangeSetV1,
    pub reasons: Vec<String>,
}

pub fn merge_preview_conservative(
    local: &SyncMergeChangeSetV1,
    remote: &SyncMergeChangeSetV1,
    now_ms: i64,
) -> AppResult<SyncMergePreviewReportV1> {
    let local_norm = normalize_change_set(local, "local")?;
    let remote_norm = normalize_change_set(remote, "remote")?;

    let local_objects: BTreeSet<String> = local_norm.object_hashes.iter().cloned().collect();
    let remote_objects: BTreeSet<String> = remote_norm.object_hashes.iter().cloned().collect();
    let overlap_objects: Vec<String> = local_objects
        .intersection(&remote_objects)
        .cloned()
        .collect();

    let local_overlays: BTreeSet<String> = local_norm.lineage_overlay_ids.iter().cloned().collect();
    let remote_overlays: BTreeSet<String> =
        remote_norm.lineage_overlay_ids.iter().cloned().collect();
    let overlap_overlays: Vec<String> = local_overlays
        .intersection(&remote_overlays)
        .cloned()
        .collect();

    let mut reasons = Vec::new();
    if !overlap_objects.is_empty() {
        reasons.push("object_hash_overlap".to_string());
    }
    if !overlap_overlays.is_empty() {
        reasons.push("lineage_overlay_overlap".to_string());
    }
    reasons.sort();

    Ok(SyncMergePreviewReportV1 {
        schema_version: 1,
        merge_policy: "conservative_v1".to_string(),
        safe: reasons.is_empty(),
        generated_at_ms: now_ms,
        local: local_norm,
        remote: remote_norm,
        overlap: SyncMergeChangeSetV1 {
            object_hashes: overlap_objects,
            lineage_overlay_ids: overlap_overlays,
        },
        reasons,
    })
}

pub fn ensure_conservative_merge_safe(report: &SyncMergePreviewReportV1) -> AppResult<()> {
    if report.safe {
        return Ok(());
    }

    Err(AppError::new(
        "KC_SYNC_MERGE_NOT_SAFE",
        "sync",
        "conservative auto-merge rejected due to overlapping change sets",
        false,
        serde_json::json!({
            "schema_version": report.schema_version,
            "merge_policy": report.merge_policy,
            "reasons": report.reasons,
            "overlap": report.overlap
        }),
    ))
}

fn normalize_change_set(
    input: &SyncMergeChangeSetV1,
    side: &str,
) -> AppResult<SyncMergeChangeSetV1> {
    let mut object_hashes = BTreeSet::new();
    for hash in &input.object_hashes {
        validate_blake3_prefixed(hash).map_err(|e| {
            AppError::new(
                "KC_SYNC_MERGE_PRECONDITION_FAILED",
                "sync",
                "sync merge preview input has invalid object hash",
                false,
                serde_json::json!({
                    "side": side,
                    "hash": hash,
                    "source_code": e.code
                }),
            )
        })?;
        object_hashes.insert(hash.clone());
    }

    let mut lineage_overlay_ids = BTreeSet::new();
    for overlay_id in &input.lineage_overlay_ids {
        if overlay_id.is_empty() {
            return Err(AppError::new(
                "KC_SYNC_MERGE_PRECONDITION_FAILED",
                "sync",
                "sync merge preview input has empty lineage overlay id",
                false,
                serde_json::json!({ "side": side }),
            ));
        }
        lineage_overlay_ids.insert(overlay_id.clone());
    }

    Ok(SyncMergeChangeSetV1 {
        object_hashes: object_hashes.into_iter().collect(),
        lineage_overlay_ids: lineage_overlay_ids.into_iter().collect(),
    })
}
