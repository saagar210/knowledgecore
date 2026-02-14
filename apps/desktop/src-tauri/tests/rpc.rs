use apps_desktop_tauri::rpc::{
    ingest_inbox_start_rpc, ingest_inbox_stop_rpc, jobs_list_rpc, vault_init_rpc, vault_open_rpc,
    IngestInboxStartReq, IngestInboxStopReq, JobsListReq, RpcResponse, VaultInitReq, VaultOpenReq,
};
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
