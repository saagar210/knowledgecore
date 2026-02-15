use kc_core::db::open_db;
use kc_core::trust::{trust_device_init, trust_device_verify};
use kc_core::trust_identity::{
    expected_cert_chain_hash, trust_device_enroll, trust_device_verify_chain, trust_identity_complete,
    trust_identity_start, trust_provider_add, trust_provider_disable, trust_provider_list,
    verified_author_identity,
};
use kc_core::trust_policy::{trust_provider_policy_set, trust_session_revoke};
use kc_core::vault::vault_init;

fn setup_verified_device_with_identity(
    conn: &rusqlite::Connection,
    provider_id: &str,
    now_ms: i64,
) -> String {
    let device = trust_device_init(conn, "workstation", "tester", now_ms).expect("trust init");
    trust_device_verify(
        conn,
        &device.device_id,
        &device.fingerprint,
        "tester",
        now_ms + 1,
    )
    .expect("trust verify");

    trust_identity_start(conn, provider_id, now_ms + 2).expect("identity start");
    trust_identity_complete(conn, provider_id, "sub:alice@example.com", now_ms + 3)
        .expect("identity complete");
    device.device_id
}

#[test]
fn trust_identity_start_and_complete_persist_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let started = trust_identity_start(&conn, "default", 120).expect("identity start");
    assert_eq!(started.provider_id, "default");
    assert!(started.authorization_url.contains("state="));
    assert!(started.state.starts_with("blake3:"));

    let completed = trust_identity_complete(&conn, "default", "sub:bob@example.com", 130)
        .expect("identity complete");
    assert_eq!(completed.provider_id, "default");
    assert_eq!(completed.subject, "bob@example.com");
    assert!(completed.claim_subset_json.contains("\"sub\":\"bob@example.com\""));
    assert!(completed.expires_at_ms > completed.issued_at_ms);
}

#[test]
fn trust_device_enroll_and_verify_chain_resolves_author_identity() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let device_id = setup_verified_device_with_identity(&conn, "default", 120);
    let enrolled = trust_device_enroll(&conn, "default", &device_id, 140).expect("enroll");
    assert_eq!(enrolled.device_id, device_id);

    let verified = trust_device_verify_chain(&conn, &device_id, 141).expect("verify chain");
    assert_eq!(verified.verified_at_ms, Some(141));

    let author = verified_author_identity(&conn).expect("author identity");
    assert_eq!(author.device_id, device_id);
    assert_eq!(author.cert_id, verified.cert_id);
    assert_eq!(author.cert_chain_hash, verified.cert_chain_hash);
}

#[test]
fn trust_device_verify_chain_rejects_tampered_chain_hash() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let device_id = setup_verified_device_with_identity(&conn, "default", 120);
    let enrolled = trust_device_enroll(&conn, "default", &device_id, 140).expect("enroll");
    conn.execute(
        "UPDATE device_certificates SET cert_chain_hash=?1 WHERE cert_id=?2",
        [
            "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            enrolled.cert_id.as_str(),
        ],
    )
    .expect("tamper chain hash");

    let err =
        trust_device_verify_chain(&conn, &device_id, 141).expect_err("chain verify must fail");
    assert_eq!(err.code, "KC_TRUST_CERT_CHAIN_INVALID");
}

#[test]
fn expected_chain_hash_is_stable() {
    let h1 = expected_cert_chain_hash("cert-1", "device-1", "fp-1");
    let h2 = expected_cert_chain_hash("cert-1", "device-1", "fp-1");
    assert_eq!(h1, h2);
    assert!(h1.starts_with("blake3:"));
}

#[test]
fn trust_provider_disable_blocks_identity_complete() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    trust_provider_add(
        &conn,
        "corp",
        "https://corp.example/oidc",
        "kc-desktop:corp",
        "https://corp.example/oidc/jwks",
        100,
    )
    .expect("add provider");
    trust_provider_disable(&conn, "corp", 110).expect("disable provider");

    let err =
        trust_identity_complete(&conn, "corp", "sub:alice@example.com", 120).expect_err("blocked");
    assert_eq!(err.code, "KC_TRUST_PROVIDER_DISABLED");
}

#[test]
fn trust_provider_policy_required_claims_are_enforced() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    trust_provider_add(
        &conn,
        "corp",
        "https://corp.example/oidc",
        "kc-desktop:corp",
        "https://corp.example/oidc/jwks",
        100,
    )
    .expect("add provider");
    trust_provider_policy_set(
        &conn,
        "corp",
        0,
        r#"{"aud":"kc-desktop:corp","iss":"https://corp.example/oidc","sub":"alice@example.com"}"#,
        101,
    )
    .expect("set policy");

    let ok = trust_identity_complete(&conn, "corp", "sub:alice@example.com", 102).expect("accept");
    assert_eq!(ok.subject, "alice@example.com");

    let err = trust_identity_complete(&conn, "corp", "sub:bob@example.com", 103)
        .expect_err("policy mismatch");
    assert_eq!(err.code, "KC_TRUST_PROVIDER_POLICY_INVALID");
}

#[test]
fn trust_session_revocation_has_precedence_over_session_selection() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let device_id = setup_verified_device_with_identity(&conn, "default", 120);
    let first =
        trust_identity_complete(&conn, "default", "sub:alice@example.com", 130).expect("first");
    let second =
        trust_identity_complete(&conn, "default", "sub:bob@example.com", 131).expect("second");

    trust_session_revoke(&conn, &second.session_id, "tester", 132).expect("revoke");
    let enrolled = trust_device_enroll(&conn, "default", &device_id, 133).expect("enroll");
    assert_eq!(enrolled.subject, first.subject);
}

#[test]
fn trust_provider_list_is_deterministic() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    trust_provider_add(
        &conn,
        "zeta",
        "https://zeta.example/oidc",
        "kc-desktop:zeta",
        "https://zeta.example/oidc/jwks",
        100,
    )
    .expect("add zeta");
    trust_provider_add(
        &conn,
        "alpha",
        "https://alpha.example/oidc",
        "kc-desktop:alpha",
        "https://alpha.example/oidc/jwks",
        101,
    )
    .expect("add alpha");

    let first = trust_provider_list(&conn).expect("list 1");
    let second = trust_provider_list(&conn).expect("list 2");
    assert_eq!(first, second);
    let ids: Vec<String> = first.into_iter().map(|p| p.provider_id).collect();
    assert_eq!(ids, vec!["alpha".to_string(), "zeta".to_string()]);
}
