use kc_core::sync_merge::{
    ensure_conservative_merge_safe, ensure_conservative_plus_v2_merge_safe,
    merge_preview_conservative, merge_preview_with_policy_v2, SyncMergeChangeSetV1,
    SyncMergeContextV2,
};

#[test]
fn sync_merge_preview_normalizes_and_dedupes_deterministically() {
    let local = SyncMergeChangeSetV1 {
        object_hashes: vec![
            "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        ],
        lineage_overlay_ids: vec![
            "overlay-z".to_string(),
            "overlay-a".to_string(),
            "overlay-a".to_string(),
        ],
    };
    let remote = SyncMergeChangeSetV1::default();

    let preview = merge_preview_conservative(&local, &remote, 123).expect("preview");
    assert!(preview.safe);
    assert_eq!(preview.schema_version, 1);
    assert_eq!(preview.merge_policy, "conservative_v1");
    assert_eq!(preview.generated_at_ms, 123);
    assert_eq!(
        preview.local.object_hashes,
        vec![
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
        ]
    );
    assert_eq!(
        preview.local.lineage_overlay_ids,
        vec!["overlay-a".to_string(), "overlay-z".to_string()]
    );
    assert!(preview.reasons.is_empty());
    assert!(preview.overlap.object_hashes.is_empty());
    assert!(preview.overlap.lineage_overlay_ids.is_empty());
}

#[test]
fn sync_merge_preview_reports_overlap_and_unsafe_decision() {
    let local = SyncMergeChangeSetV1 {
        object_hashes: vec![
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
        ],
        lineage_overlay_ids: vec!["overlay-1".to_string(), "overlay-2".to_string()],
    };
    let remote = SyncMergeChangeSetV1 {
        object_hashes: vec![
            "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
        ],
        lineage_overlay_ids: vec!["overlay-2".to_string(), "overlay-9".to_string()],
    };

    let preview = merge_preview_conservative(&local, &remote, 999).expect("preview");
    assert!(!preview.safe);
    assert_eq!(
        preview.reasons,
        vec![
            "lineage_overlay_overlap".to_string(),
            "object_hash_overlap".to_string(),
        ]
    );
    assert_eq!(
        preview.overlap.object_hashes,
        vec!["blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string()]
    );
    assert_eq!(
        preview.overlap.lineage_overlay_ids,
        vec!["overlay-2".to_string()]
    );

    let err = ensure_conservative_merge_safe(&preview).expect_err("unsafe merge");
    assert_eq!(err.code, "KC_SYNC_MERGE_NOT_SAFE");
}

#[test]
fn sync_merge_preview_rejects_invalid_input_hash() {
    let local = SyncMergeChangeSetV1 {
        object_hashes: vec!["blake3:not-hex".to_string()],
        lineage_overlay_ids: vec![],
    };
    let remote = SyncMergeChangeSetV1::default();

    let err = merge_preview_conservative(&local, &remote, 100).expect_err("invalid hash");
    assert_eq!(err.code, "KC_SYNC_MERGE_PRECONDITION_FAILED");
}

#[test]
fn sync_merge_preview_v2_supports_disjoint_safe_merge() {
    let local = SyncMergeChangeSetV1 {
        object_hashes: vec![
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        ],
        lineage_overlay_ids: vec!["overlay-a".to_string()],
    };
    let remote = SyncMergeChangeSetV1 {
        object_hashes: vec![
            "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
        ],
        lineage_overlay_ids: vec!["overlay-b".to_string()],
    };

    let report = merge_preview_with_policy_v2(
        &local,
        &remote,
        &SyncMergeContextV2::default(),
        "conservative_plus_v2",
        555,
    )
    .expect("v2 preview");
    assert!(report.safe);
    assert_eq!(report.schema_version, 2);
    assert_eq!(report.merge_policy, "conservative_plus_v2");
    assert!(report.reasons.is_empty());
    assert_eq!(
        report.decision_trace,
        vec![
            "policy=conservative_plus_v2".to_string(),
            "local.object_hashes=1".to_string(),
            "local.lineage_overlay_ids=1".to_string(),
            "remote.object_hashes=1".to_string(),
            "remote.lineage_overlay_ids=1".to_string(),
            "overlap.object_hashes=0".to_string(),
            "overlap.lineage_overlay_ids=0".to_string(),
            "trust_chain_mismatch=false".to_string(),
            "lock_conflict=false".to_string(),
        ]
    );
}

#[test]
fn sync_merge_preview_v2_flags_trust_and_lock_conflicts_with_specific_codes() {
    let local = SyncMergeChangeSetV1::default();
    let remote = SyncMergeChangeSetV1::default();
    let trust_conflict = SyncMergeContextV2 {
        trust_chain_mismatch: true,
        lock_conflict: false,
    };
    let trust_report =
        merge_preview_with_policy_v2(&local, &remote, &trust_conflict, "conservative_plus_v2", 1)
            .expect("trust report");
    assert!(!trust_report.safe);
    assert_eq!(
        trust_report.reasons,
        vec!["trust_chain_mismatch".to_string()]
    );
    let trust_err = ensure_conservative_plus_v2_merge_safe(&trust_report).expect_err("trust err");
    assert_eq!(trust_err.code, "KC_SYNC_MERGE_TRUST_CONFLICT");

    let lock_conflict = SyncMergeContextV2 {
        trust_chain_mismatch: false,
        lock_conflict: true,
    };
    let lock_report =
        merge_preview_with_policy_v2(&local, &remote, &lock_conflict, "conservative_plus_v2", 2)
            .expect("lock report");
    assert!(!lock_report.safe);
    assert_eq!(
        lock_report.reasons,
        vec!["lineage_lock_conflict".to_string()]
    );
    let lock_err = ensure_conservative_plus_v2_merge_safe(&lock_report).expect_err("lock err");
    assert_eq!(lock_err.code, "KC_SYNC_MERGE_LOCK_CONFLICT");
}

#[test]
fn sync_merge_preview_v2_rejects_unknown_policy() {
    let local = SyncMergeChangeSetV1::default();
    let remote = SyncMergeChangeSetV1::default();
    let err = merge_preview_with_policy_v2(
        &local,
        &remote,
        &SyncMergeContextV2::default(),
        "unsupported_policy",
        100,
    )
    .expect_err("unsupported policy");
    assert_eq!(err.code, "KC_SYNC_MERGE_POLICY_UNSUPPORTED");
}
