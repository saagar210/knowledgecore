use kc_core::db::open_db;
use kc_core::lineage_policy::{
    ensure_lineage_policy_allows, lineage_policy_add, lineage_policy_bind, lineage_policy_decision,
    lineage_policy_list,
};
use kc_core::vault::vault_init;

#[test]
fn lineage_policy_list_is_deterministic_by_priority_policy_and_subject() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    lineage_policy_add(
        &conn,
        "allow-b",
        "allow",
        r#"{"action":"lineage.overlay.write","doc_id_prefix":"doc_"}"#,
        "tests",
        10,
    )
    .expect("add allow b");
    lineage_policy_add(
        &conn,
        "allow-a",
        "allow",
        r#"{"action":"lineage.overlay.write","doc_id_prefix":"doc_"}"#,
        "tests",
        11,
    )
    .expect("add allow a");
    lineage_policy_add(
        &conn,
        "deny-a",
        "deny",
        r#"{"action":"lineage.overlay.write","doc_id_prefix":"doc_"}"#,
        "tests",
        12,
    )
    .expect("add deny a");

    lineage_policy_bind(&conn, "subject-z", "allow-b", "tests", 20).expect("bind 1");
    lineage_policy_bind(&conn, "subject-a", "allow-a", "tests", 21).expect("bind 2");
    lineage_policy_bind(&conn, "subject-a", "deny-a", "tests", 22).expect("bind 3");

    let listed_a = lineage_policy_list(&conn).expect("list a");
    let listed_b = lineage_policy_list(&conn).expect("list b");
    assert_eq!(listed_a, listed_b);

    // deny policies have lower numeric priority than allow policies.
    assert_eq!(listed_a[0].effect, "deny");
    assert_eq!(listed_a[0].subject_id, "subject-a");
}

#[test]
fn lineage_policy_decision_enforces_deny_overrides_allow() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    lineage_policy_add(
        &conn,
        "allow-all-overlay",
        "allow",
        r#"{"action":"lineage.overlay.write","doc_id_prefix":"doc_"}"#,
        "tests",
        10,
    )
    .expect("add allow");
    lineage_policy_add(
        &conn,
        "deny-doc-special",
        "deny",
        r#"{"action":"lineage.overlay.write","doc_id_prefix":"doc_special"}"#,
        "tests",
        10,
    )
    .expect("add deny");

    lineage_policy_bind(&conn, "alice", "allow-all-overlay", "tests", 11).expect("bind allow");
    lineage_policy_bind(&conn, "alice", "deny-doc-special", "tests", 12).expect("bind deny");

    let allowed =
        lineage_policy_decision(&conn, "alice", "lineage.overlay.write", Some("doc_alpha"))
            .expect("decision allowed");
    assert!(allowed.allowed);
    assert_eq!(allowed.reason, "policy_allow");

    let denied = lineage_policy_decision(
        &conn,
        "alice",
        "lineage.overlay.write",
        Some("doc_special_1"),
    )
    .expect("decision denied");
    assert!(!denied.allowed);
    assert_eq!(denied.reason, "policy_deny");
    assert_eq!(denied.matched_effect.as_deref(), Some("deny"));
}

#[test]
fn ensure_lineage_policy_allows_denies_when_no_matching_allow_and_writes_audit() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    lineage_policy_add(
        &conn,
        "allow-suffix",
        "allow",
        r#"{"action":"lineage.overlay.write","doc_id_prefix":"doc_target_"}"#,
        "tests",
        10,
    )
    .expect("add allow");
    lineage_policy_bind(&conn, "bob", "allow-suffix", "tests", 11).expect("bind allow");

    let err = ensure_lineage_policy_allows(
        &conn,
        "bob",
        "lineage.overlay.write",
        Some("doc_other_1"),
        100,
    )
    .expect_err("no matching allow should deny");
    assert_eq!(err.code, "KC_LINEAGE_PERMISSION_DENIED");

    ensure_lineage_policy_allows(
        &conn,
        "bob",
        "lineage.overlay.write",
        Some("doc_target_1"),
        101,
    )
    .expect("matching allow should pass");

    let rows: Vec<(i64, String, i64)> = {
        let mut stmt = conn
            .prepare(
                "SELECT ts_ms, reason, allowed FROM lineage_policy_audit ORDER BY ts_ms ASC, audit_id ASC",
            )
            .expect("prepare audit query");
        let iter = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .expect("query audit rows");
        let mut out = Vec::new();
        for row in iter {
            out.push(row.expect("decode audit row"));
        }
        out
    };
    assert_eq!(
        rows,
        vec![
            (100, "no_matching_allow_policy".to_string(), 0),
            (101, "policy_allow".to_string(), 1),
        ]
    );
}

#[test]
fn lineage_policy_decision_supports_doc_suffix_and_subject_prefix_conditions() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    lineage_policy_add(
        &conn,
        "allow-team-release-docs",
        "allow",
        r#"{"action":"lineage.overlay.write","doc_id_suffix":"_release","subject_id_prefix":"team-"}"#,
        "tests",
        10,
    )
    .expect("add allow");
    lineage_policy_bind(&conn, "team-alice", "allow-team-release-docs", "tests", 11)
        .expect("bind team allow");
    lineage_policy_bind(&conn, "user-bob", "allow-team-release-docs", "tests", 12)
        .expect("bind non-team allow");

    let allowed = lineage_policy_decision(
        &conn,
        "team-alice",
        "lineage.overlay.write",
        Some("doc_1_release"),
    )
    .expect("team release decision");
    assert!(allowed.allowed);
    assert_eq!(allowed.reason, "policy_allow");

    let denied_suffix = lineage_policy_decision(
        &conn,
        "team-alice",
        "lineage.overlay.write",
        Some("doc_1_draft"),
    )
    .expect("suffix mismatch decision");
    assert!(!denied_suffix.allowed);
    assert_eq!(denied_suffix.reason, "no_matching_allow_policy");

    let denied_subject = lineage_policy_decision(
        &conn,
        "user-bob",
        "lineage.overlay.write",
        Some("doc_1_release"),
    )
    .expect("subject mismatch decision");
    assert!(!denied_subject.allowed);
    assert_eq!(denied_subject.reason, "no_matching_allow_policy");
}

#[test]
fn lineage_policy_add_rejects_empty_v4_condition_fields() {
    let root = tempfile::tempdir().expect("tempdir").keep();
    vault_init(&root, "demo", 1).expect("vault init");
    let conn = open_db(&root.join("db/knowledge.sqlite")).expect("open db");

    let err_suffix = lineage_policy_add(
        &conn,
        "invalid-suffix",
        "allow",
        r#"{"action":"lineage.overlay.write","doc_id_suffix":""}"#,
        "tests",
        10,
    )
    .expect_err("empty suffix must fail");
    assert_eq!(err_suffix.code, "KC_LINEAGE_POLICY_CONDITION_INVALID");

    let err_subject = lineage_policy_add(
        &conn,
        "invalid-subject-prefix",
        "allow",
        r#"{"action":"lineage.overlay.write","subject_id_prefix":""}"#,
        "tests",
        11,
    )
    .expect_err("empty subject prefix must fail");
    assert_eq!(err_subject.code, "KC_LINEAGE_POLICY_CONDITION_INVALID");
}
