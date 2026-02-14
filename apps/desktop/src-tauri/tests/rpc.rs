use apps_desktop_tauri::commands;
use apps_desktop_tauri::rpc::{
    ingest_inbox_start_rpc, ingest_inbox_stop_rpc, jobs_list_rpc, lineage_overlay_add_rpc,
    lineage_overlay_list_rpc, lineage_overlay_remove_rpc, lineage_query_rpc, lineage_query_v2_rpc,
    lineage_lock_acquire_rpc, lineage_lock_release_rpc, lineage_lock_status_rpc,
    sync_merge_preview_rpc, sync_pull_rpc, sync_push_rpc, sync_status_rpc,
    vault_encryption_enable_rpc, vault_encryption_migrate_rpc, vault_encryption_status_rpc,
    vault_init_rpc, vault_lock_rpc, vault_lock_status_rpc, vault_open_rpc,
    vault_recovery_generate_rpc, vault_recovery_status_rpc, vault_recovery_verify_rpc,
    vault_unlock_rpc, IngestInboxStartReq, IngestInboxStopReq, JobsListReq, LineageLockAcquireReq,
    LineageLockReleaseReq, LineageLockStatusReq, LineageOverlayAddReq, LineageOverlayListReq,
    LineageOverlayRemoveReq, LineageQueryReq, LineageQueryV2Req,
    RpcResponse, SyncMergePreviewReq, SyncPullReq, SyncPushReq, SyncStatusReq,
    VaultEncryptionEnableReq, VaultEncryptionMigrateReq, VaultEncryptionStatusReq, VaultInitReq,
    VaultLockReq, VaultLockStatusReq, VaultOpenReq, VaultRecoveryGenerateReq,
    VaultRecoveryStatusReq, VaultRecoveryVerifyReq, VaultUnlockReq,
};
use kc_core::app_error::AppError;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn rpc_envelope_success_shape() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let response = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });

    match response {
        RpcResponse::Ok { ref data } => {
            assert!(!data.vault_id.is_empty());
        }
        RpcResponse::Err { .. } => panic!("expected success response"),
    }

    let serialized = serde_json::to_value(&response).expect("serialize rpc");
    assert_eq!(serialized.get("ok").and_then(|v| v.as_bool()), Some(true));
    assert!(serialized.get("data").is_some());
    assert!(serialized.get("error").is_none());
}

#[test]
fn rpc_envelope_error_shape() {
    let error = AppError::new("KC_RPC_FAIL", "rpc", "failed", true, serde_json::json!({}));
    let response: RpcResponse<()> = RpcResponse::err(error.clone());

    let serialized = serde_json::to_value(&response).expect("serialize rpc");
    assert_eq!(serialized.get("ok").and_then(|v| v.as_bool()), Some(false));
    assert!(serialized.get("data").is_none());
    assert_eq!(
        serialized
            .get("error")
            .and_then(|v| v.get("code"))
            .and_then(|v| v.as_str()),
        Some(error.code.as_str())
    );

    let round_trip: RpcResponse<()> = serde_json::from_value(serialized).expect("deserialize rpc");
    match round_trip {
        RpcResponse::Err { error: e } => assert_eq!(e.code, "KC_RPC_FAIL"),
        RpcResponse::Ok { .. } => panic!("expected error response"),
    }
}

#[test]
fn rpc_vault_open_and_jobs_list() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let opened = vault_open_rpc(VaultOpenReq {
        vault_path: root.to_string_lossy().to_string(),
    });
    match opened {
        RpcResponse::Ok { data } => assert_eq!(data.vault_slug, "demo"),
        RpcResponse::Err { error } => panic!("vault open failed: {}", error.code),
    }

    let jobs = jobs_list_rpc(JobsListReq {
        vault_path: root.to_string_lossy().to_string(),
    });
    match jobs {
        RpcResponse::Ok { data } => assert!(data.jobs.is_empty()),
        RpcResponse::Err { error } => panic!("jobs list failed: {}", error.code),
    }
}

#[test]
fn rpc_vault_lock_status_unlock_and_lock_round_trip() {
    let root = tempfile::tempdir().expect("tempdir").keep();

    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let status_before = vault_lock_status_rpc(VaultLockStatusReq {
        vault_path: root.to_string_lossy().to_string(),
    });
    match status_before {
        RpcResponse::Ok { data } => {
            assert!(!data.db_encryption_enabled);
            assert!(data.unlocked);
        }
        RpcResponse::Err { error } => panic!("lock status failed: {}", error.code),
    }

    let unlocked = vault_unlock_rpc(VaultUnlockReq {
        vault_path: root.to_string_lossy().to_string(),
        passphrase: "test-passphrase".to_string(),
    });
    match unlocked {
        RpcResponse::Ok { data } => assert!(data.status.unlocked),
        RpcResponse::Err { error } => panic!("unlock failed: {}", error.code),
    }

    let locked = vault_lock_rpc(VaultLockReq {
        vault_path: root.to_string_lossy().to_string(),
    });
    match locked {
        RpcResponse::Ok { data } => assert!(data.status.unlocked),
        RpcResponse::Err { error } => panic!("lock failed: {}", error.code),
    }
}

#[test]
fn rpc_vault_encryption_status_enable_and_migrate() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let input = root.join("note.txt");
    std::fs::write(&input, b"hello").expect("write input");

    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let started = ingest_inbox_start_rpc(IngestInboxStartReq {
        vault_path: root.to_string_lossy().to_string(),
        file_path: input.to_string_lossy().to_string(),
        source_kind: "notes".to_string(),
        now_ms: 2,
    });
    match started {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("inbox start failed: {}", error.code),
    }

    let status_before = vault_encryption_status_rpc(VaultEncryptionStatusReq {
        vault_path: root.to_string_lossy().to_string(),
    });
    match status_before {
        RpcResponse::Ok { data } => {
            assert!(!data.enabled);
            assert_eq!(data.objects_total, 1);
            assert_eq!(data.objects_encrypted, 0);
        }
        RpcResponse::Err { error } => panic!("status failed: {}", error.code),
    }

    let enabled = vault_encryption_enable_rpc(VaultEncryptionEnableReq {
        vault_path: root.to_string_lossy().to_string(),
        passphrase: "test-passphrase".to_string(),
    });
    match enabled {
        RpcResponse::Ok { data } => assert!(data.status.enabled),
        RpcResponse::Err { error } => panic!("enable failed: {}", error.code),
    }

    let migrated = vault_encryption_migrate_rpc(VaultEncryptionMigrateReq {
        vault_path: root.to_string_lossy().to_string(),
        passphrase: "test-passphrase".to_string(),
        now_ms: 3,
    });
    match migrated {
        RpcResponse::Ok { data } => {
            assert_eq!(data.migrated_objects, 1);
            assert_eq!(data.status.objects_encrypted, 1);
        }
        RpcResponse::Err { error } => panic!("migrate failed: {}", error.code),
    }
}

#[test]
fn rpc_vault_recovery_status_generate_and_verify() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let output = root.join("recovery-output");

    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let status_before = vault_recovery_status_rpc(VaultRecoveryStatusReq {
        vault_path: root.to_string_lossy().to_string(),
    });
    match status_before {
        RpcResponse::Ok { data } => {
            assert!(data.last_bundle_path.is_none());
        }
        RpcResponse::Err { error } => panic!("recovery status failed: {}", error.code),
    }

    let generated = vault_recovery_generate_rpc(VaultRecoveryGenerateReq {
        vault_path: root.to_string_lossy().to_string(),
        output_dir: output.to_string_lossy().to_string(),
        passphrase: "vault-passphrase".to_string(),
        now_ms: 100,
    });
    let (bundle_path, phrase) = match generated {
        RpcResponse::Ok { data } => {
            assert_eq!(data.manifest.schema_version, 1);
            (data.bundle_path, data.recovery_phrase)
        }
        RpcResponse::Err { error } => panic!("recovery generate failed: {}", error.code),
    };

    let verified = vault_recovery_verify_rpc(VaultRecoveryVerifyReq {
        vault_path: root.to_string_lossy().to_string(),
        bundle_path,
        recovery_phrase: phrase,
    });
    match verified {
        RpcResponse::Ok { data } => assert_eq!(data.manifest.schema_version, 1),
        RpcResponse::Err { error } => panic!("recovery verify failed: {}", error.code),
    }
}

#[test]
fn rpc_sync_status_and_push() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let sync_target = root.join("sync-target");
    let input = root.join("note-sync.txt");
    std::fs::write(&input, b"hello sync").expect("write input");

    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let started = ingest_inbox_start_rpc(IngestInboxStartReq {
        vault_path: root.to_string_lossy().to_string(),
        file_path: input.to_string_lossy().to_string(),
        source_kind: "notes".to_string(),
        now_ms: 2,
    });
    match started {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("ingest failed: {}", error.code),
    }

    let status_before = sync_status_rpc(SyncStatusReq {
        vault_path: root.to_string_lossy().to_string(),
        target_path: sync_target.to_string_lossy().to_string(),
    });
    match status_before {
        RpcResponse::Ok { data } => assert!(data.remote_head.is_none()),
        RpcResponse::Err { error } => panic!("sync status failed: {}", error.code),
    }

    let pushed = sync_push_rpc(SyncPushReq {
        vault_path: root.to_string_lossy().to_string(),
        target_path: sync_target.to_string_lossy().to_string(),
        now_ms: 3,
    });
    match pushed {
        RpcResponse::Ok { data } => assert!(!data.snapshot_id.is_empty()),
        RpcResponse::Err { error } => panic!("sync push failed: {}", error.code),
    }
}

#[test]
fn rpc_sync_supports_s3_uri_targets_via_emulation() {
    let _guard = env_lock().lock().expect("env lock");
    let root = tempfile::tempdir().expect("tempdir").keep();
    let pull_root = tempfile::tempdir().expect("pull tempdir").keep();
    let emulated_s3 = root.join("emulated-s3");
    std::env::set_var(
        "KC_SYNC_S3_EMULATE_ROOT",
        emulated_s3.to_string_lossy().as_ref(),
    );
    std::env::set_var("KC_VAULT_PASSPHRASE", "rpc-sync-passphrase");

    let input = root.join("note-sync-s3.txt");
    std::fs::write(&input, b"hello sync s3").expect("write input");

    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let pull_init = vault_init_rpc(VaultInitReq {
        vault_path: pull_root.to_string_lossy().to_string(),
        vault_slug: "pull-demo".to_string(),
        now_ms: 1,
    });
    match pull_init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("pull vault init failed: {}", error.code),
    }

    let started = ingest_inbox_start_rpc(IngestInboxStartReq {
        vault_path: root.to_string_lossy().to_string(),
        file_path: input.to_string_lossy().to_string(),
        source_kind: "notes".to_string(),
        now_ms: 2,
    });
    match started {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("ingest failed: {}", error.code),
    }

    let target_uri = "s3://demo-bucket/kc";
    let pushed = sync_push_rpc(SyncPushReq {
        vault_path: root.to_string_lossy().to_string(),
        target_path: target_uri.to_string(),
        now_ms: 3,
    });
    let pushed_snapshot_id = match pushed {
        RpcResponse::Ok { data } => data.snapshot_id,
        RpcResponse::Err { error } => panic!("sync push failed: {}", error.code),
    };

    let status = sync_status_rpc(SyncStatusReq {
        vault_path: root.to_string_lossy().to_string(),
        target_path: target_uri.to_string(),
    });
    match status {
        RpcResponse::Ok { data } => {
            assert_eq!(data.target_path, target_uri);
            assert_eq!(
                data.remote_head.map(|h| h.snapshot_id),
                Some(pushed_snapshot_id.clone())
            );
        }
        RpcResponse::Err { error } => panic!("sync status failed: {}", error.code),
    }

    let pulled = sync_pull_rpc(SyncPullReq {
        vault_path: pull_root.to_string_lossy().to_string(),
        target_path: target_uri.to_string(),
        auto_merge: Some("conservative".to_string()),
        now_ms: 4,
    });
    match pulled {
        RpcResponse::Ok { data } => assert_eq!(data.snapshot_id, pushed_snapshot_id),
        RpcResponse::Err { error } => panic!("sync pull failed: {}", error.code),
    }

    let preview = sync_merge_preview_rpc(SyncMergePreviewReq {
        vault_path: root.to_string_lossy().to_string(),
        target_path: target_uri.to_string(),
        now_ms: 5,
    });
    match preview {
        RpcResponse::Ok { data } => {
            assert_eq!(data.target_path, target_uri);
            assert_eq!(data.report.merge_policy, "conservative_v1");
        }
        RpcResponse::Err { error } => panic!("sync merge preview failed: {}", error.code),
    }

    std::env::remove_var("KC_VAULT_PASSPHRASE");
    std::env::remove_var("KC_SYNC_S3_EMULATE_ROOT");
}

#[test]
fn rpc_lineage_query_is_deterministic_and_sorted() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let input = root.join("note-lineage.txt");
    std::fs::write(&input, b"lineage seed").expect("write input");

    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let started = ingest_inbox_start_rpc(IngestInboxStartReq {
        vault_path: root.to_string_lossy().to_string(),
        file_path: input.to_string_lossy().to_string(),
        source_kind: "notes".to_string(),
        now_ms: 2,
    });
    let seed_doc_id = match started {
        RpcResponse::Ok { data } => data.doc_id,
        RpcResponse::Err { error } => panic!("ingest failed: {}", error.code),
    };

    let req = LineageQueryReq {
        vault_path: root.to_string_lossy().to_string(),
        seed_doc_id,
        depth: 2,
        now_ms: 3,
    };
    let res_a = lineage_query_rpc(req);
    let req_b = LineageQueryReq {
        vault_path: root.to_string_lossy().to_string(),
        seed_doc_id: match &res_a {
            RpcResponse::Ok { data } => data.seed_doc_id.clone(),
            RpcResponse::Err { .. } => "missing".to_string(),
        },
        depth: 2,
        now_ms: 3,
    };
    let res_b = lineage_query_rpc(req_b);
    assert_eq!(
        serde_json::to_value(&res_a).expect("serialize a"),
        serde_json::to_value(&res_b).expect("serialize b")
    );

    match res_a {
        RpcResponse::Ok { data } => {
            let node_keys: Vec<(String, String)> = data
                .nodes
                .iter()
                .map(|n| (n.kind.clone(), n.node_id.clone()))
                .collect();
            let mut sorted_node_keys = node_keys.clone();
            sorted_node_keys.sort();
            assert_eq!(node_keys, sorted_node_keys);

            let edge_keys: Vec<(String, String, String)> = data
                .edges
                .iter()
                .map(|e| {
                    (
                        e.from_node_id.clone(),
                        e.to_node_id.clone(),
                        e.relation.clone(),
                    )
                })
                .collect();
            let mut sorted_edge_keys = edge_keys.clone();
            sorted_edge_keys.sort();
            assert_eq!(edge_keys, sorted_edge_keys);
        }
        RpcResponse::Err { error } => panic!("lineage query failed: {}", error.code),
    }
}

#[test]
fn rpc_lineage_v2_overlay_round_trip_is_deterministic() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let input = root.join("note-lineage-v2.txt");
    std::fs::write(&input, b"lineage seed v2").expect("write input");

    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let started = ingest_inbox_start_rpc(IngestInboxStartReq {
        vault_path: root.to_string_lossy().to_string(),
        file_path: input.to_string_lossy().to_string(),
        source_kind: "notes".to_string(),
        now_ms: 2,
    });
    let seed_doc_id = match started {
        RpcResponse::Ok { data } => data.doc_id,
        RpcResponse::Err { error } => panic!("ingest failed: {}", error.code),
    };

    let acquired = lineage_lock_acquire_rpc(LineageLockAcquireReq {
        vault_path: root.to_string_lossy().to_string(),
        doc_id: seed_doc_id.clone(),
        owner: "desktop-test".to_string(),
        now_ms: 3,
    });
    let lock_token = match acquired {
        RpcResponse::Ok { data } => data.lease.token,
        RpcResponse::Err { error } => panic!("lineage lock acquire failed: {}", error.code),
    };

    let lock_status = lineage_lock_status_rpc(LineageLockStatusReq {
        vault_path: root.to_string_lossy().to_string(),
        doc_id: seed_doc_id.clone(),
        now_ms: 4,
    });
    match lock_status {
        RpcResponse::Ok { data } => {
            assert!(data.held);
            assert_eq!(data.owner.as_deref(), Some("desktop-test"));
        }
        RpcResponse::Err { error } => panic!("lineage lock status failed: {}", error.code),
    }

    let added = lineage_overlay_add_rpc(LineageOverlayAddReq {
        vault_path: root.to_string_lossy().to_string(),
        doc_id: seed_doc_id.clone(),
        from_node_id: format!("doc:{seed_doc_id}"),
        to_node_id: "note:overlay-1".to_string(),
        relation: "supports".to_string(),
        evidence: "manual".to_string(),
        lock_token: lock_token.clone(),
        created_at_ms: 5,
        created_by: Some("desktop-test".to_string()),
    });
    let overlay_id = match added {
        RpcResponse::Ok { data } => {
            assert_eq!(data.overlay.doc_id, seed_doc_id);
            data.overlay.overlay_id
        }
        RpcResponse::Err { error } => panic!("overlay add failed: {}", error.code),
    };

    let listed = lineage_overlay_list_rpc(LineageOverlayListReq {
        vault_path: root.to_string_lossy().to_string(),
        doc_id: seed_doc_id.clone(),
    });
    match listed {
        RpcResponse::Ok { data } => {
            assert_eq!(data.overlays.len(), 1);
            assert_eq!(data.overlays[0].overlay_id, overlay_id);
        }
        RpcResponse::Err { error } => panic!("overlay list failed: {}", error.code),
    }

    let req = LineageQueryV2Req {
        vault_path: root.to_string_lossy().to_string(),
        seed_doc_id: seed_doc_id.clone(),
        depth: 2,
        now_ms: 6,
    };
    let res_a = lineage_query_v2_rpc(req);
    let res_b = lineage_query_v2_rpc(LineageQueryV2Req {
        vault_path: root.to_string_lossy().to_string(),
        seed_doc_id: seed_doc_id.clone(),
        depth: 2,
        now_ms: 6,
    });
    assert_eq!(
        serde_json::to_value(&res_a).expect("serialize lineage a"),
        serde_json::to_value(&res_b).expect("serialize lineage b")
    );

    match res_a {
        RpcResponse::Ok { data } => {
            let has_overlay_edge = data.edges.iter().any(|edge| edge.origin == "overlay");
            assert!(has_overlay_edge);

            let keys: Vec<(String, String, String, String, String)> = data
                .edges
                .iter()
                .map(|e| {
                    (
                        e.from_node_id.clone(),
                        e.to_node_id.clone(),
                        e.relation.clone(),
                        e.evidence.clone(),
                        e.origin.clone(),
                    )
                })
                .collect();
            let mut sorted = keys.clone();
            sorted.sort();
            assert_eq!(keys, sorted);
        }
        RpcResponse::Err { error } => panic!("lineage query v2 failed: {}", error.code),
    }

    let removed = lineage_overlay_remove_rpc(LineageOverlayRemoveReq {
        vault_path: root.to_string_lossy().to_string(),
        overlay_id: overlay_id.clone(),
        lock_token: lock_token.clone(),
        now_ms: 7,
    });
    match removed {
        RpcResponse::Ok { data } => assert_eq!(data.removed_overlay_id, overlay_id),
        RpcResponse::Err { error } => panic!("overlay remove failed: {}", error.code),
    }

    let released = lineage_lock_release_rpc(LineageLockReleaseReq {
        vault_path: root.to_string_lossy().to_string(),
        doc_id: seed_doc_id.clone(),
        token: lock_token,
    });
    match released {
        RpcResponse::Ok { data } => assert!(data.released),
        RpcResponse::Err { error } => panic!("lineage lock release failed: {}", error.code),
    }

    let listed_after_remove = lineage_overlay_list_rpc(LineageOverlayListReq {
        vault_path: root.to_string_lossy().to_string(),
        doc_id: seed_doc_id,
    });
    match listed_after_remove {
        RpcResponse::Ok { data } => assert!(data.overlays.is_empty()),
        RpcResponse::Err { error } => panic!("overlay list after remove failed: {}", error.code),
    }
}

#[test]
fn rpc_ingest_inbox_start_and_stop() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let input = root.join("note.txt");
    std::fs::write(&input, b"hello").expect("write input");

    let init = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    match init {
        RpcResponse::Ok { .. } => {}
        RpcResponse::Err { error } => panic!("vault init failed: {}", error.code),
    }

    let started = ingest_inbox_start_rpc(IngestInboxStartReq {
        vault_path: root.to_string_lossy().to_string(),
        file_path: input.to_string_lossy().to_string(),
        source_kind: "notes".to_string(),
        now_ms: 2,
    });

    let job_id = match started {
        RpcResponse::Ok { data } => {
            assert!(!data.doc_id.is_empty());
            data.job_id
        }
        RpcResponse::Err { error } => panic!("inbox start failed: {}", error.code),
    };

    let stopped = ingest_inbox_stop_rpc(IngestInboxStopReq {
        vault_path: root.to_string_lossy().to_string(),
        job_id,
    });
    match stopped {
        RpcResponse::Ok { data } => assert!(data.stopped),
        RpcResponse::Err { error } => panic!("inbox stop failed: {}", error.code),
    }
}

#[test]
fn tauri_command_wrappers_use_rpc_envelope_contract() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let via_command = commands::vault_init(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });
    let via_rpc = vault_init_rpc(VaultInitReq {
        vault_path: root.to_string_lossy().to_string(),
        vault_slug: "demo".to_string(),
        now_ms: 1,
    });

    let command_json = serde_json::to_value(via_command).expect("serialize command response");
    let rpc_json = serde_json::to_value(via_rpc).expect("serialize rpc response");
    assert_eq!(
        command_json.get("ok").and_then(|v| v.as_bool()),
        rpc_json.get("ok").and_then(|v| v.as_bool())
    );
    assert!(command_json.get("data").is_some());
}
