use kc_core::canon_json::to_canonical_bytes;
use kc_core::db::open_db;
use kc_core::ingest::ingest_bytes;
use kc_core::lineage::{lineage_lock_acquire, lineage_overlay_add, lineage_overlay_remove};
use kc_core::lineage_governance::{
    ensure_lineage_permission, lineage_lock_acquire_scope, lineage_lock_release_scope,
    lineage_lock_scope_status, lineage_permission_decision, lineage_role_grant, lineage_role_list,
    lineage_role_revoke,
};
use kc_core::lineage_policy::{
    ensure_lineage_policy_allows, lineage_policy_add, lineage_policy_bind,
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
    lineage_policy_add(
        &conn,
        "allow-overlay-governance",
        "allow",
        r#"{"action":"lineage.overlay.write"}"#,
        "tests",
        12,
    )
    .expect("add allow policy");
    lineage_policy_bind(&conn, "owner-a", "allow-overlay-governance", "tests", 12)
        .expect("bind allow policy");
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

#[test]
fn lineage_policy_audit_details_are_deterministic_and_canonical() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    lineage_policy_add(
        &conn,
        "allow-doc-prefix",
        "allow",
        r#"{"action":"lineage.overlay.write","doc_id_prefix":"doc_allow_"}"#,
        "tests",
        10,
    )
    .expect("add allow policy");
    lineage_policy_add(
        &conn,
        "deny-doc-prefix",
        "deny",
        r#"{"action":"lineage.overlay.write","doc_id_prefix":"doc_deny_"}"#,
        "tests",
        11,
    )
    .expect("add deny policy");
    lineage_policy_bind(&conn, "owner-a", "allow-doc-prefix", "tests", 12)
        .expect("bind allow policy");
    lineage_policy_bind(&conn, "owner-a", "deny-doc-prefix", "tests", 13)
        .expect("bind deny policy");

    ensure_lineage_policy_allows(
        &conn,
        "owner-a",
        "lineage.overlay.write",
        Some("doc_allow_001"),
        200,
    )
    .expect("allow decision");

    let denied = ensure_lineage_policy_allows(
        &conn,
        "owner-a",
        "lineage.overlay.write",
        Some("doc_deny_001"),
        201,
    )
    .expect_err("deny decision");
    assert_eq!(denied.code, "KC_LINEAGE_POLICY_DENY_ENFORCED");

    let rows: Vec<(i64, i64, String, String)> = {
        let mut stmt = conn
            .prepare(
                "SELECT ts_ms, allowed, reason, details_json
                 FROM lineage_policy_audit
                 ORDER BY ts_ms ASC, audit_id ASC",
            )
            .expect("prepare policy audit query");
        let iter = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .expect("query policy audit rows");

        let mut out = Vec::new();
        for row in iter {
            out.push(row.expect("decode policy audit row"));
        }
        out
    };

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, 200);
    assert_eq!(rows[0].1, 1);
    assert_eq!(rows[0].2, "policy_allow");

    let allow_id: String = conn
        .query_row(
            "SELECT policy_id FROM lineage_policies WHERE policy_name='allow-doc-prefix'",
            [],
            |row| row.get(0),
        )
        .expect("load allow policy id");
    let expected_allow_details = serde_json::json!({
        "action": "lineage.overlay.write",
        "doc_id": "doc_allow_001",
        "matched_effect": "allow",
        "matched_policy_id": allow_id,
        "matched_policy_name": "allow-doc-prefix",
        "reason": "policy_allow",
        "subject_id": "owner-a"
    });
    assert_eq!(
        rows[0].3,
        String::from_utf8(
            to_canonical_bytes(&expected_allow_details).expect("canonical allow details"),
        )
        .expect("utf8 allow details")
    );

    assert_eq!(rows[1].0, 201);
    assert_eq!(rows[1].1, 0);
    assert_eq!(rows[1].2, "policy_deny");
    let deny_id: String = conn
        .query_row(
            "SELECT policy_id FROM lineage_policies WHERE policy_name='deny-doc-prefix'",
            [],
            |row| row.get(0),
        )
        .expect("load deny policy id");
    let expected_deny_details = serde_json::json!({
        "action": "lineage.overlay.write",
        "doc_id": "doc_deny_001",
        "matched_effect": "deny",
        "matched_policy_id": deny_id,
        "matched_policy_name": "deny-doc-prefix",
        "reason": "policy_deny",
        "subject_id": "owner-a"
    });
    assert_eq!(
        rows[1].3,
        String::from_utf8(
            to_canonical_bytes(&expected_deny_details).expect("canonical deny details"),
        )
        .expect("utf8 deny details")
    );
}
