use kc_core::db::open_db;
use kc_core::ingest::ingest_bytes;
use kc_core::lineage::{lineage_lock_acquire, lineage_overlay_add, lineage_overlay_remove};
use kc_core::lineage_governance::{
    ensure_lineage_permission, lineage_lock_acquire_scope, lineage_lock_release_scope,
    lineage_lock_scope_status, lineage_permission_decision, lineage_role_grant, lineage_role_list,
    lineage_role_revoke,
};
use kc_core::object_store::ObjectStore;
use kc_core::vault::vault_init;
use rusqlite::params;

#[test]
fn lineage_role_list_is_sorted_by_rank_then_subject() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    lineage_role_grant(&conn, "zeta", "viewer", "tests", 1).expect("grant zeta viewer");
    lineage_role_grant(&conn, "alpha", "editor", "tests", 2).expect("grant alpha editor");
    lineage_role_grant(&conn, "beta", "admin", "tests", 3).expect("grant beta admin");

    let listed_a = lineage_role_list(&conn).expect("role list a");
    let listed_b = lineage_role_list(&conn).expect("role list b");
    assert_eq!(listed_a, listed_b);

    let summary: Vec<(String, String, i64)> = listed_a
        .into_iter()
        .map(|b| (b.subject_id, b.role_name, b.role_rank))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("beta".to_string(), "admin".to_string(), 10),
            ("alpha".to_string(), "editor".to_string(), 20),
            ("zeta".to_string(), "viewer".to_string(), 30),
        ]
    );
}

#[test]
fn lineage_permission_evaluation_uses_role_rank_precedence() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    conn.execute(
        "INSERT INTO lineage_roles(role_name, role_rank, description) VALUES (?1, ?2, ?3)",
        params!["blocked", 5i64, "deny overlay writes"],
    )
    .expect("insert blocked role");
    conn.execute(
        "INSERT INTO lineage_permissions(role_name, action, allowed) VALUES (?1, ?2, ?3)",
        params!["blocked", "lineage.overlay.write", 0i64],
    )
    .expect("insert blocked permission");

    lineage_role_grant(&conn, "actor", "editor", "tests", 1).expect("grant editor");
    lineage_role_grant(&conn, "actor", "blocked", "tests", 2).expect("grant blocked");

    let denied = lineage_permission_decision(&conn, "actor", "lineage.overlay.write")
        .expect("permission decision");
    assert!(!denied.allowed);
    assert_eq!(denied.matched_role.as_deref(), Some("blocked"));
    assert_eq!(denied.matched_rank, Some(5));

    let err = ensure_lineage_permission(&conn, "actor", "lineage.overlay.write", Some("doc:1"))
        .expect_err("permission should deny");
    assert_eq!(err.code, "KC_LINEAGE_PERMISSION_DENIED");

    lineage_role_revoke(&conn, "actor", "blocked").expect("revoke blocked");
    ensure_lineage_permission(&conn, "actor", "lineage.overlay.write", Some("doc:1"))
        .expect("editor should allow after revoke");
}

#[test]
fn lineage_scope_lock_round_trip_and_validation() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    let lease = lineage_lock_acquire_scope(&conn, "doc", "doc-1", "owner-a", 100)
        .expect("acquire scope lock");
    assert_eq!(lease.scope_kind, "doc");
    assert_eq!(lease.scope_value, "doc-1");

    let status = lineage_lock_scope_status(&conn, "doc", "doc-1", 101).expect("scope status");
    assert!(status.held);
    assert_eq!(status.owner.as_deref(), Some("owner-a"));

    let held_err = lineage_lock_acquire_scope(&conn, "doc", "doc-1", "owner-b", 102)
        .expect_err("competing lock must fail");
    assert_eq!(held_err.code, "KC_LINEAGE_LOCK_HELD");

    let invalid_release = lineage_lock_release_scope(&conn, "doc", "doc-1", "bad-token")
        .expect_err("invalid token must fail");
    assert_eq!(invalid_release.code, "KC_LINEAGE_LOCK_INVALID");

    lineage_lock_release_scope(&conn, "doc", "doc-1", &lease.token).expect("release scope lock");
    let released = lineage_lock_scope_status(&conn, "doc", "doc-1", 103).expect("released status");
    assert!(!released.held);

    let scope_err = lineage_lock_acquire_scope(&conn, "invalid", "doc-1", "owner-a", 200)
        .expect_err("invalid scope must fail");
    assert_eq!(scope_err.code, "KC_LINEAGE_SCOPE_INVALID");
}

#[test]
fn lineage_overlay_mutation_requires_rbac_permission() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    let store = ObjectStore::new(root.join("store/objects"));
    let ingested = ingest_bytes(
        &conn,
        &store,
        b"lineage governance",
        "text/plain",
        "notes",
        1,
        None,
        1,
    )
    .expect("ingest");
    let doc_id = ingested.doc_id.0;
    let lock = lineage_lock_acquire(&conn, &doc_id, "owner-a", 10).expect("acquire lock");

    let denied = lineage_overlay_add(
        &conn,
        &doc_id,
        &format!("doc:{doc_id}"),
        "note:overlay",
        "supports",
        "rbac",
        &lock.token,
        11,
        "owner-a",
    )
    .expect_err("overlay add should require permission");
    assert_eq!(denied.code, "KC_LINEAGE_PERMISSION_DENIED");

    lineage_role_grant(&conn, "owner-a", "editor", "tests", 12).expect("grant editor");
    let added = lineage_overlay_add(
        &conn,
        &doc_id,
        &format!("doc:{doc_id}"),
        "note:overlay",
        "supports",
        "rbac",
        &lock.token,
        13,
        "owner-a",
    )
    .expect("overlay add with permission");
    lineage_overlay_remove(&conn, &added.overlay_id, &lock.token, 14)
        .expect("overlay remove with owner permission");
}
