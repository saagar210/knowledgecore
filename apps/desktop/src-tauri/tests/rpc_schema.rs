use apps_desktop_tauri::rpc::{
    AskQuestionReq, LineageOverlayAddReq, LineageOverlayListReq, LineageOverlayRemoveReq,
    LineageQueryReq, LineageQueryV2Req, SearchQueryReq, SyncPullReq, SyncPushReq, SyncStatusReq,
    VaultEncryptionEnableReq, VaultEncryptionMigrateReq, VaultEncryptionStatusReq, VaultInitReq,
    VaultLockReq, VaultLockStatusReq, VaultRecoveryGenerateReq, VaultRecoveryStatusReq,
    VaultRecoveryVerifyReq, VaultUnlockReq,
};

#[test]
fn rpc_schema_requires_now_ms_on_deterministic_requests() {
    let missing_vault_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "vault_slug": "demo"
    });
    let missing_search_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "query": "foo"
    });
    let missing_ask_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "question": "What happened?"
    });

    assert!(serde_json::from_value::<VaultInitReq>(missing_vault_now).is_err());
    assert!(serde_json::from_value::<SearchQueryReq>(missing_search_now).is_err());
    assert!(serde_json::from_value::<AskQuestionReq>(missing_ask_now).is_err());
}

#[test]
fn rpc_schema_rejects_unknown_fields() {
    let req = serde_json::json!({
        "vault_path": "/tmp/vault",
        "query": "foo",
        "now_ms": 123,
        "extra": "nope"
    });
    assert!(serde_json::from_value::<SearchQueryReq>(req).is_err());

    let invalid_status = serde_json::json!({
        "vault_path": "/tmp/vault",
        "extra": "nope"
    });
    assert!(serde_json::from_value::<VaultEncryptionStatusReq>(invalid_status).is_err());

    let invalid_enable = serde_json::json!({
        "vault_path": "/tmp/vault",
        "passphrase": "secret",
        "extra": "nope"
    });
    assert!(serde_json::from_value::<VaultEncryptionEnableReq>(invalid_enable).is_err());

    let invalid_unlock = serde_json::json!({
        "vault_path": "/tmp/vault",
        "passphrase": "secret",
        "extra": "nope"
    });
    assert!(serde_json::from_value::<VaultUnlockReq>(invalid_unlock).is_err());

    let invalid_lock = serde_json::json!({
        "vault_path": "/tmp/vault",
        "extra": "nope"
    });
    assert!(serde_json::from_value::<VaultLockReq>(invalid_lock).is_err());
}

#[test]
fn rpc_schema_requires_now_ms_for_encryption_migrate() {
    let missing_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "passphrase": "secret"
    });
    assert!(serde_json::from_value::<VaultEncryptionMigrateReq>(missing_now).is_err());

    let missing_sync_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "target_path": "/tmp/target"
    });
    assert!(serde_json::from_value::<SyncPushReq>(missing_sync_now.clone()).is_err());
    assert!(serde_json::from_value::<SyncPullReq>(missing_sync_now).is_err());
}

#[test]
fn rpc_schema_requires_now_ms_for_recovery_generate() {
    let missing_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "output_dir": "/tmp/out",
        "passphrase": "secret"
    });
    assert!(serde_json::from_value::<VaultRecoveryGenerateReq>(missing_now).is_err());
}

#[test]
fn rpc_schema_recovery_requests_validate_shapes() {
    let status = serde_json::json!({
        "vault_path": "/tmp/vault"
    });
    assert!(serde_json::from_value::<VaultRecoveryStatusReq>(status).is_ok());

    let generate = serde_json::json!({
        "vault_path": "/tmp/vault",
        "output_dir": "/tmp/out",
        "passphrase": "secret",
        "now_ms": 123
    });
    assert!(serde_json::from_value::<VaultRecoveryGenerateReq>(generate).is_ok());

    let verify = serde_json::json!({
        "vault_path": "/tmp/vault",
        "bundle_path": "/tmp/out/recovery",
        "recovery_phrase": "abcd-efgh"
    });
    assert!(serde_json::from_value::<VaultRecoveryVerifyReq>(verify).is_ok());
}

#[test]
fn rpc_schema_recovery_rejects_unknown_fields() {
    let invalid = serde_json::json!({
        "vault_path": "/tmp/vault",
        "extra": "nope"
    });
    assert!(serde_json::from_value::<VaultRecoveryStatusReq>(invalid).is_err());
}

#[test]
fn rpc_schema_sync_rejects_unknown_fields() {
    let invalid_status = serde_json::json!({
        "vault_path": "/tmp/vault",
        "target_path": "/tmp/target",
        "extra": "nope"
    });
    assert!(serde_json::from_value::<SyncStatusReq>(invalid_status).is_err());
}

#[test]
fn rpc_schema_sync_accepts_uri_targets() {
    let status = serde_json::json!({
        "vault_path": "/tmp/vault",
        "target_path": "s3://demo-bucket/kc"
    });
    assert!(serde_json::from_value::<SyncStatusReq>(status).is_ok());

    let push = serde_json::json!({
        "vault_path": "/tmp/vault",
        "target_path": "s3://demo-bucket/kc",
        "now_ms": 123
    });
    assert!(serde_json::from_value::<SyncPushReq>(push.clone()).is_ok());
    assert!(serde_json::from_value::<SyncPullReq>(push).is_ok());
}

#[test]
fn rpc_schema_lock_requests_validate_shapes() {
    let lock_status = serde_json::json!({
        "vault_path": "/tmp/vault"
    });
    assert!(serde_json::from_value::<VaultLockStatusReq>(lock_status).is_ok());

    let unlock = serde_json::json!({
        "vault_path": "/tmp/vault",
        "passphrase": "secret"
    });
    assert!(serde_json::from_value::<VaultUnlockReq>(unlock).is_ok());
}

#[test]
fn rpc_schema_requires_now_ms_for_lineage_query() {
    let missing_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "seed_doc_id": "doc-1",
        "depth": 1
    });
    assert!(serde_json::from_value::<LineageQueryReq>(missing_now).is_err());
}

#[test]
fn rpc_schema_lineage_rejects_unknown_fields() {
    let invalid = serde_json::json!({
        "vault_path": "/tmp/vault",
        "seed_doc_id": "doc-1",
        "depth": 1,
        "now_ms": 123,
        "extra": "nope"
    });
    assert!(serde_json::from_value::<LineageQueryReq>(invalid).is_err());
}

#[test]
fn rpc_schema_requires_now_ms_for_lineage_query_v2() {
    let missing_now = serde_json::json!({
        "vault_path": "/tmp/vault",
        "seed_doc_id": "doc-1",
        "depth": 1
    });
    assert!(serde_json::from_value::<LineageQueryV2Req>(missing_now).is_err());
}

#[test]
fn rpc_schema_lineage_overlay_add_requires_created_at_ms() {
    let missing_created_at = serde_json::json!({
        "vault_path": "/tmp/vault",
        "doc_id": "doc-1",
        "from_node_id": "doc:doc-1",
        "to_node_id": "chunk:c1",
        "relation": "supports",
        "evidence": "manual"
    });
    assert!(serde_json::from_value::<LineageOverlayAddReq>(missing_created_at).is_err());
}

#[test]
fn rpc_schema_lineage_overlay_requests_validate_shapes() {
    let add = serde_json::json!({
        "vault_path": "/tmp/vault",
        "doc_id": "doc-1",
        "from_node_id": "doc:doc-1",
        "to_node_id": "chunk:c1",
        "relation": "supports",
        "evidence": "manual",
        "created_at_ms": 123,
        "created_by": "user"
    });
    assert!(serde_json::from_value::<LineageOverlayAddReq>(add).is_ok());

    let list = serde_json::json!({
        "vault_path": "/tmp/vault",
        "doc_id": "doc-1"
    });
    assert!(serde_json::from_value::<LineageOverlayListReq>(list).is_ok());

    let remove = serde_json::json!({
        "vault_path": "/tmp/vault",
        "overlay_id": "blake3:abcd"
    });
    assert!(serde_json::from_value::<LineageOverlayRemoveReq>(remove).is_ok());
}

#[test]
fn rpc_schema_lineage_overlay_rejects_unknown_fields() {
    let invalid = serde_json::json!({
        "vault_path": "/tmp/vault",
        "overlay_id": "blake3:abcd",
        "extra": "nope"
    });
    assert!(serde_json::from_value::<LineageOverlayRemoveReq>(invalid).is_err());
}
