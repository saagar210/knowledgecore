use kc_core::sync_merge::{
    ensure_conservative_merge_safe, merge_preview_conservative, SyncMergeChangeSetV1,
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
