use crate::app_error::{AppError, AppResult};
use crate::hashing::validate_blake3_prefixed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const POLICY_CONSERVATIVE_V1: &str = "conservative_v1";
const POLICY_CONSERVATIVE_PLUS_V2: &str = "conservative_plus_v2";

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_trace: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SyncMergeContextV2 {
    pub trust_chain_mismatch: bool,
    pub lock_conflict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncMergePreviewReportV2 {
    pub schema_version: i64,
    pub merge_policy: String,
    pub safe: bool,
    pub generated_at_ms: i64,
    pub local: SyncMergeChangeSetV1,
    pub remote: SyncMergeChangeSetV1,
    pub overlap: SyncMergeChangeSetV1,
    pub reasons: Vec<String>,
    pub decision_trace: Vec<String>,
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
        merge_policy: POLICY_CONSERVATIVE_V1.to_string(),
        safe: reasons.is_empty(),
        generated_at_ms: now_ms,
        local: local_norm,
        remote: remote_norm,
        overlap: SyncMergeChangeSetV1 {
            object_hashes: overlap_objects,
            lineage_overlay_ids: overlap_overlays,
        },
        reasons,
        decision_trace: None,
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

pub fn merge_preview_with_policy_v2(
    local: &SyncMergeChangeSetV1,
    remote: &SyncMergeChangeSetV1,
    ctx: &SyncMergeContextV2,
    policy: &str,
    now_ms: i64,
) -> AppResult<SyncMergePreviewReportV2> {
    match policy {
        POLICY_CONSERVATIVE_PLUS_V2 => {
            merge_preview_conservative_plus_v2(local, remote, ctx, now_ms)
        }
        other => Err(AppError::new(
            "KC_SYNC_MERGE_POLICY_UNSUPPORTED",
            "sync",
            "unsupported sync merge policy",
            false,
            serde_json::json!({
                "policy": other,
                "supported": [POLICY_CONSERVATIVE_PLUS_V2]
            }),
        )),
    }
}

pub fn merge_preview_conservative_plus_v2(
    local: &SyncMergeChangeSetV1,
    remote: &SyncMergeChangeSetV1,
    ctx: &SyncMergeContextV2,
    now_ms: i64,
) -> AppResult<SyncMergePreviewReportV2> {
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
    if ctx.trust_chain_mismatch {
        reasons.push("trust_chain_mismatch".to_string());
    }
    if ctx.lock_conflict {
        reasons.push("lineage_lock_conflict".to_string());
    }
    reasons.sort();

    let decision_trace = vec![
        format!("policy={}", POLICY_CONSERVATIVE_PLUS_V2),
        format!("local.object_hashes={}", local_norm.object_hashes.len()),
        format!(
            "local.lineage_overlay_ids={}",
            local_norm.lineage_overlay_ids.len()
        ),
        format!("remote.object_hashes={}", remote_norm.object_hashes.len()),
        format!(
            "remote.lineage_overlay_ids={}",
            remote_norm.lineage_overlay_ids.len()
        ),
        format!("overlap.object_hashes={}", overlap_objects.len()),
        format!("overlap.lineage_overlay_ids={}", overlap_overlays.len()),
        format!("trust_chain_mismatch={}", ctx.trust_chain_mismatch),
        format!("lock_conflict={}", ctx.lock_conflict),
    ];

    Ok(SyncMergePreviewReportV2 {
        schema_version: 2,
        merge_policy: POLICY_CONSERVATIVE_PLUS_V2.to_string(),
        safe: reasons.is_empty(),
        generated_at_ms: now_ms,
        local: local_norm,
        remote: remote_norm,
        overlap: SyncMergeChangeSetV1 {
            object_hashes: overlap_objects,
            lineage_overlay_ids: overlap_overlays,
        },
        reasons,
        decision_trace,
    })
}

pub fn ensure_conservative_plus_v2_merge_safe(report: &SyncMergePreviewReportV2) -> AppResult<()> {
    if report.safe {
        return Ok(());
    }

    let code = if report
        .reasons
        .iter()
        .any(|reason| reason == "trust_chain_mismatch")
    {
        "KC_SYNC_MERGE_TRUST_CONFLICT"
    } else if report
        .reasons
        .iter()
        .any(|reason| reason == "lineage_lock_conflict")
    {
        "KC_SYNC_MERGE_LOCK_CONFLICT"
    } else {
        "KC_SYNC_MERGE_NOT_SAFE"
    };

    Err(AppError::new(
        code,
        "sync",
        "conservative_plus_v2 auto-merge rejected by safety policy",
        false,
        serde_json::json!({
            "schema_version": report.schema_version,
            "merge_policy": report.merge_policy,
            "reasons": report.reasons,
            "overlap": report.overlap,
            "decision_trace": report.decision_trace,
        }),
    ))
}

impl From<SyncMergePreviewReportV2> for SyncMergePreviewReportV1 {
    fn from(report: SyncMergePreviewReportV2) -> Self {
        Self {
            schema_version: report.schema_version,
            merge_policy: report.merge_policy,
            safe: report.safe,
            generated_at_ms: report.generated_at_ms,
            local: report.local,
            remote: report.remote,
            overlap: report.overlap,
            reasons: report.reasons,
            decision_trace: Some(report.decision_trace),
        }
    }
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
