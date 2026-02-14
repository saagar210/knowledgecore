use apps_desktop_tauri::rpc::{
    ingest_inbox_start_rpc, ingest_inbox_stop_rpc, jobs_list_rpc, vault_encryption_enable_rpc,
    vault_encryption_migrate_rpc, vault_encryption_status_rpc, vault_init_rpc, vault_open_rpc,
    IngestInboxStartReq, IngestInboxStopReq, JobsListReq, RpcResponse, VaultEncryptionEnableReq,
    VaultEncryptionMigrateReq, VaultEncryptionStatusReq, VaultInitReq, VaultOpenReq, SyncPushReq,
    SyncStatusReq, sync_push_rpc, sync_status_rpc, LineageQueryReq, lineage_query_rpc,
};
use apps_desktop_tauri::commands;
use kc_core::app_error::AppError;

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
            let node_keys: Vec<(String, String)> =
                data.nodes.iter().map(|n| (n.kind.clone(), n.node_id.clone())).collect();
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

#[cfg(not(feature = "phase_l_preview"))]
#[test]
fn rpc_preview_is_disabled_by_default() {
    assert!(!apps_desktop_tauri::rpc::phase_l_preview_enabled());
}

#[cfg(feature = "phase_l_preview")]
#[test]
fn rpc_preview_is_enabled_with_feature() {
    assert!(apps_desktop_tauri::rpc::phase_l_preview_enabled());
}

#[cfg(feature = "phase_l_preview")]
#[test]
fn rpc_preview_status_returns_draft_capabilities() {
    use apps_desktop_tauri::rpc::{preview_status_rpc, PreviewStatusReq};
    let response = preview_status_rpc(PreviewStatusReq {});
    match response {
        RpcResponse::Ok { data } => {
            assert_eq!(data.status, "draft");
            let ordered: Vec<String> = data.capabilities.into_iter().map(|c| c.capability).collect();
            assert_eq!(ordered, vec!["encryption", "lineage", "sync", "zip_packaging"]);
        }
        RpcResponse::Err { error } => panic!("preview status failed: {}", error.code),
    }
}

#[cfg(feature = "phase_l_preview")]
#[test]
fn rpc_preview_capability_returns_placeholder_error() {
    use apps_desktop_tauri::rpc::{preview_capability_rpc, PreviewCapabilityReq};
    let response = preview_capability_rpc(PreviewCapabilityReq {
        name: "sync".to_string(),
    });
    match response {
        RpcResponse::Err { error } => assert_eq!(error.code, "KC_DRAFT_SYNC_NOT_IMPLEMENTED"),
        RpcResponse::Ok { .. } => panic!("expected draft error response"),
    }
}
