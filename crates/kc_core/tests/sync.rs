use kc_core::db::open_db;
use kc_core::object_store::ObjectStore;
use kc_core::sync::{
    sync_pull, sync_pull_target, sync_push, sync_push_target, sync_status, sync_status_target,
    SyncHeadV1,
};
use kc_core::vault::vault_init;
use std::sync::{Mutex, OnceLock};

fn insert_object(
    conn: &rusqlite::Connection,
    vault_root: &std::path::Path,
    bytes: &[u8],
    event_id: i64,
) {
    let store = ObjectStore::new(vault_root.join("store/objects"));
    store.put_bytes(conn, bytes, event_id).expect("put object");
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn sync_push_writes_head_and_status() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let target_root = root.join("sync-target");

    vault_init(&vault_root, "demo", 1).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    insert_object(&conn, &vault_root, b"one", 1);

    let pushed = sync_push(&conn, &vault_root, &target_root, 100).expect("sync push");
    assert!(target_root.join("head.json").exists());
    assert!(target_root
        .join("snapshots")
        .join(&pushed.snapshot_id)
        .exists());

    let status = sync_status(&conn, &target_root).expect("sync status");
    assert_eq!(
        status.seen_remote_snapshot_id,
        Some(pushed.snapshot_id.clone())
    );
    assert_eq!(
        status.last_applied_manifest_hash,
        Some(pushed.manifest_hash.clone())
    );
}

#[test]
fn sync_pull_applies_remote_snapshot() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let source_vault = root.join("vault_source");
    let target_vault = root.join("vault_target");
    let sync_target = root.join("sync-target");

    vault_init(&source_vault, "source", 1).expect("source init");
    vault_init(&target_vault, "target", 1).expect("target init");

    let conn_source = open_db(&source_vault.join("db/knowledge.sqlite")).expect("source db");
    insert_object(&conn_source, &source_vault, b"source-object", 1);
    sync_push(&conn_source, &source_vault, &sync_target, 100).expect("push");

    let conn_target = open_db(&target_vault.join("db/knowledge.sqlite")).expect("target db");
    let pulled = sync_pull(&conn_target, &target_vault, &sync_target, 101).expect("pull");

    let post_conn = open_db(&target_vault.join("db/knowledge.sqlite")).expect("post db");
    let object_count: i64 = post_conn
        .query_row("SELECT COUNT(*) FROM objects", [], |row| row.get(0))
        .expect("count objects");
    assert!(object_count >= 1);
    assert!(!pulled.snapshot_id.is_empty());
}

#[test]
fn sync_push_conflict_emits_artifact() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let target_root = root.join("sync-target");

    vault_init(&vault_root, "demo", 1).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    insert_object(&conn, &vault_root, b"baseline", 1);

    let first = sync_push(&conn, &vault_root, &target_root, 100).expect("first push");

    insert_object(&conn, &vault_root, b"local-change", 2);

    let remote_head = SyncHeadV1 {
        schema_version: 1,
        snapshot_id: format!("{}-remote", first.snapshot_id),
        manifest_hash: "blake3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            .to_string(),
        created_at_ms: 200,
        trust: None,
        author_device_id: None,
        author_fingerprint: None,
        author_signature: None,
    };
    std::fs::write(
        target_root.join("head.json"),
        serde_json::to_vec(&remote_head).expect("head json"),
    )
    .expect("write head");

    let err = sync_push(&conn, &vault_root, &target_root, 201).expect_err("expected conflict");
    assert_eq!(err.code, "KC_SYNC_CONFLICT");

    let conflict_path = err
        .details
        .get("conflict_artifact")
        .and_then(|v| v.as_str())
        .expect("conflict path");
    assert!(std::path::Path::new(conflict_path).exists());
}

#[test]
fn sync_target_wrappers_support_file_uri() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let target_root = root.join("sync-target");
    let target_uri = format!("file://{}", target_root.display());

    vault_init(&vault_root, "demo", 1).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    insert_object(&conn, &vault_root, b"uri", 1);

    let pushed = sync_push_target(&conn, &vault_root, &target_uri, 100).expect("sync push target");
    assert!(!pushed.snapshot_id.is_empty());

    let status = sync_status_target(&conn, &target_uri).expect("sync status target");
    assert_eq!(
        status.seen_remote_snapshot_id,
        Some(pushed.snapshot_id.clone())
    );

    let pulled = sync_pull_target(&conn, &vault_root, &target_uri, 101).expect("sync pull target");
    assert_eq!(pulled.snapshot_id, pushed.snapshot_id);
}

#[test]
fn sync_target_wrappers_support_s3_uri_with_emulation() {
    let _guard = env_lock().lock().expect("env lock");
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let pull_vault_root = root.join("vault_pull");
    let emulated_s3 = root.join("emulated-s3");
    std::env::set_var(
        "KC_SYNC_S3_EMULATE_ROOT",
        emulated_s3.to_string_lossy().as_ref(),
    );
    std::env::set_var("KC_VAULT_PASSPHRASE", "sync-passphrase");

    vault_init(&vault_root, "demo", 1).expect("vault init");
    vault_init(&pull_vault_root, "pull-demo", 1).expect("pull vault init");

    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    let conn_pull = open_db(&pull_vault_root.join("db/knowledge.sqlite")).expect("open pull db");
    insert_object(&conn, &vault_root, b"s3-sync-object", 1);

    let target_uri = "s3://demo-bucket/kc";
    let pushed = sync_push_target(&conn, &vault_root, target_uri, 100).expect("s3 push");
    assert!(emulated_s3.join("demo-bucket/kc/head.json").exists());
    assert!(emulated_s3
        .join(format!(
            "demo-bucket/kc/snapshots/{}.zip",
            pushed.snapshot_id
        ))
        .exists());

    let status = sync_status_target(&conn, target_uri).expect("s3 status");
    assert_eq!(
        status.seen_remote_snapshot_id,
        Some(pushed.snapshot_id.clone())
    );

    let pulled = sync_pull_target(&conn_pull, &pull_vault_root, target_uri, 101).expect("s3 pull");
    assert_eq!(pulled.snapshot_id, pushed.snapshot_id);

    std::env::remove_var("KC_VAULT_PASSPHRASE");
    std::env::remove_var("KC_SYNC_S3_EMULATE_ROOT");
}

#[test]
fn sync_s3_key_mismatch_hard_fails() {
    let _guard = env_lock().lock().expect("env lock");
    let root = tempfile::tempdir().expect("tempdir").keep();
    let source_vault = root.join("vault_source");
    let target_vault = root.join("vault_target");
    let emulated_s3 = root.join("emulated-s3");
    std::env::set_var(
        "KC_SYNC_S3_EMULATE_ROOT",
        emulated_s3.to_string_lossy().as_ref(),
    );

    vault_init(&source_vault, "source", 1).expect("source init");
    vault_init(&target_vault, "target", 1).expect("target init");

    let source_conn = open_db(&source_vault.join("db/knowledge.sqlite")).expect("source db");
    let target_conn = open_db(&target_vault.join("db/knowledge.sqlite")).expect("target db");
    insert_object(&source_conn, &source_vault, b"source-object", 1);

    std::env::set_var("KC_VAULT_PASSPHRASE", "alpha");
    sync_push_target(&source_conn, &source_vault, "s3://demo-bucket/kc", 200).expect("source push");

    std::env::set_var("KC_VAULT_PASSPHRASE", "beta");
    let err = sync_pull_target(&target_conn, &target_vault, "s3://demo-bucket/kc", 201)
        .expect_err("expected key mismatch");
    assert_eq!(err.code, "KC_SYNC_KEY_MISMATCH");

    std::env::remove_var("KC_VAULT_PASSPHRASE");
    std::env::remove_var("KC_SYNC_S3_EMULATE_ROOT");
}

#[test]
fn sync_s3_reports_locked_when_lock_file_is_active() {
    let _guard = env_lock().lock().expect("env lock");
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    let emulated_s3 = root.join("emulated-s3");
    std::env::set_var(
        "KC_SYNC_S3_EMULATE_ROOT",
        emulated_s3.to_string_lossy().as_ref(),
    );
    std::env::set_var("KC_VAULT_PASSPHRASE", "lock-passphrase");

    vault_init(&vault_root, "demo", 1).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");
    insert_object(&conn, &vault_root, b"payload", 1);

    let lock = serde_json::json!({
        "schema_version": 1,
        "holder": "other:999",
        "vault_id": "other-vault",
        "acquired_at_ms": 100,
        "expires_at_ms": 999_999,
        "trust_fingerprint": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });
    let lock_path = emulated_s3.join("demo-bucket/kc/locks/write.lock");
    std::fs::create_dir_all(lock_path.parent().expect("lock parent")).expect("mkdir lock parent");
    std::fs::write(&lock_path, serde_json::to_vec(&lock).expect("lock json")).expect("write lock");

    let err = sync_push_target(&conn, &vault_root, "s3://demo-bucket/kc", 123)
        .expect_err("expected locked error");
    assert_eq!(err.code, "KC_SYNC_LOCKED");

    std::env::remove_var("KC_VAULT_PASSPHRASE");
    std::env::remove_var("KC_SYNC_S3_EMULATE_ROOT");
}
