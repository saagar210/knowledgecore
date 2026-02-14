use kc_core::db::open_db;
use kc_core::object_store::ObjectStore;
use kc_core::sync::{
    sync_pull, sync_pull_target, sync_push, sync_push_target, sync_status, sync_status_target,
    SyncHeadV1,
};
use kc_core::vault::vault_init;

fn insert_object(conn: &rusqlite::Connection, vault_root: &std::path::Path, bytes: &[u8], event_id: i64) {
    let store = ObjectStore::new(vault_root.join("store/objects"));
    store.put_bytes(conn, bytes, event_id).expect("put object");
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
    assert!(
        target_root
            .join("snapshots")
            .join(&pushed.snapshot_id)
            .exists()
    );

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

    let pushed =
        sync_push_target(&conn, &vault_root, &target_uri, 100).expect("sync push target");
    assert!(!pushed.snapshot_id.is_empty());

    let status = sync_status_target(&conn, &target_uri).expect("sync status target");
    assert_eq!(status.seen_remote_snapshot_id, Some(pushed.snapshot_id.clone()));

    let pulled =
        sync_pull_target(&conn, &vault_root, &target_uri, 101).expect("sync pull target");
    assert_eq!(pulled.snapshot_id, pushed.snapshot_id);
}

#[test]
fn sync_target_wrappers_reject_s3_until_enabled() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    let vault_root = root.join("vault");
    vault_init(&vault_root, "demo", 1).expect("vault init");
    let conn = open_db(&vault_root.join("db/knowledge.sqlite")).expect("open db");

    let err = sync_push_target(&conn, &vault_root, "s3://demo-bucket/kc", 100)
        .expect_err("s3 push should be unsupported in this milestone");
    assert_eq!(err.code, "KC_SYNC_TARGET_UNSUPPORTED");
}
