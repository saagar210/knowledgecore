use kc_core::db::open_db;
use kc_core::ingest::ingest_bytes;
use kc_core::lineage::{
    lineage_lock_acquire, lineage_lock_release, lineage_lock_status, lineage_overlay_add,
    lineage_overlay_list,
};
use kc_core::lineage_governance::lineage_role_grant;
use kc_core::lineage_policy::{lineage_policy_add, lineage_policy_bind};
use kc_core::object_store::ObjectStore;
use kc_core::vault::vault_init;

fn seed_doc(conn: &rusqlite::Connection, vault_root: &std::path::Path) -> String {
    let store = ObjectStore::new(vault_root.join("store/objects"));
    let ingested = ingest_bytes(
        conn,
        &store,
        b"lineage lock seed",
        "text/plain",
        "notes",
        1,
        None,
        1,
    )
    .expect("ingest");
    ingested.doc_id.0
}

fn grant_overlay_policy(conn: &rusqlite::Connection, subject_id: &str, now_ms: i64) {
    lineage_policy_add(
        conn,
        "allow-overlay-lock-tests",
        "allow",
        r#"{"action":"lineage.overlay.write"}"#,
        "test-harness",
        now_ms,
    )
    .expect("add overlay policy");
    lineage_policy_bind(
        conn,
        subject_id,
        "allow-overlay-lock-tests",
        "test-harness",
        now_ms,
    )
    .expect("bind overlay policy");
}

#[test]
fn lineage_lock_acquire_status_and_release_round_trip() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
    let doc_id = seed_doc(&conn, &root);

    let lock = lineage_lock_acquire(&conn, &doc_id, "tester", 100).expect("acquire");
    assert_eq!(lock.doc_id, doc_id);
    assert_eq!(lock.owner, "tester");
    assert_eq!(lock.acquired_at_ms, 100);
    assert_eq!(lock.expires_at_ms, 100 + 15 * 60 * 1000);

    let status = lineage_lock_status(&conn, &doc_id, 101).expect("status held");
    assert!(status.held);
    assert_eq!(status.owner.as_deref(), Some("tester"));
    assert!(!status.expired);

    lineage_lock_release(&conn, &doc_id, &lock.token).expect("release");
    let released = lineage_lock_status(&conn, &doc_id, 102).expect("status released");
    assert!(!released.held);
    assert!(released.owner.is_none());
}

#[test]
fn lineage_lock_rejects_competing_holder_before_expiry() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
    let doc_id = seed_doc(&conn, &root);

    let _first = lineage_lock_acquire(&conn, &doc_id, "owner-a", 100).expect("first lock");
    let err = lineage_lock_acquire(&conn, &doc_id, "owner-b", 200).expect_err("must fail held");
    assert_eq!(err.code, "KC_LINEAGE_LOCK_HELD");
}

#[test]
fn lineage_overlay_mutation_requires_valid_lock_token() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
    let doc_id = seed_doc(&conn, &root);
    let doc_node = format!("doc:{}", doc_id);
    let chunk_node = "chunk:lock-1";

    let err = lineage_overlay_add(
        &conn,
        &doc_id,
        &doc_node,
        chunk_node,
        "supports",
        "manual",
        "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        100,
        "test",
    )
    .expect_err("missing valid lock should fail");
    assert_eq!(err.code, "KC_LINEAGE_LOCK_INVALID");

    let lock = lineage_lock_acquire(&conn, &doc_id, "tester", 100).expect("acquire lock");
    lineage_role_grant(&conn, "tester", "editor", "test-harness", 100).expect("grant editor");
    grant_overlay_policy(&conn, "tester", 100);
    let _added = lineage_overlay_add(
        &conn,
        &doc_id,
        &doc_node,
        chunk_node,
        "supports",
        "manual",
        &lock.token,
        101,
        "tester",
    )
    .expect("overlay add with lock");
    let listed = lineage_overlay_list(&conn, &doc_id).expect("list");
    assert_eq!(listed.len(), 1);
}

#[test]
fn lineage_overlay_mutation_rejects_expired_lock() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");
    let doc_id = seed_doc(&conn, &root);
    let doc_node = format!("doc:{}", doc_id);
    let lock = lineage_lock_acquire(&conn, &doc_id, "tester", 100).expect("acquire lock");
    lineage_role_grant(&conn, "tester", "editor", "test-harness", 100).expect("grant editor");
    grant_overlay_policy(&conn, "tester", 100);

    let err = lineage_overlay_add(
        &conn,
        &doc_id,
        &doc_node,
        "chunk:expired",
        "supports",
        "manual",
        &lock.token,
        100 + 15 * 60 * 1000 + 1,
        "tester",
    )
    .expect_err("expired lock should fail");
    assert_eq!(err.code, "KC_LINEAGE_LOCK_EXPIRED");
}
