use kc_core::db::open_db;
use kc_core::trust::{
    format_device_fingerprint, trust_device_init, trust_device_list, trust_device_verify,
    trust_events,
};
use kc_core::vault::vault_init;

#[test]
fn trust_device_init_generates_fingerprint_and_records_event() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let created = trust_device_init(&conn, "laptop", "tester", 120).expect("trust init");
    assert_eq!(created.label, "laptop");
    assert!(created.verified_at_ms.is_none());
    assert!(created.pubkey.len() >= 64);
    assert_eq!(created.fingerprint.len(), 71);
    assert_eq!(created.fingerprint.matches(':').count(), 7);

    let events = trust_events(&conn).expect("trust events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "init");
    assert_eq!(events[0].actor, "tester");
}

#[test]
fn trust_device_verify_updates_timestamp_and_logs_event() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let created = trust_device_init(&conn, "desktop", "tester", 120).expect("trust init");
    let verified = trust_device_verify(
        &conn,
        &created.device_id,
        &created.fingerprint,
        "tester",
        130,
    )
    .expect("trust verify");
    assert_eq!(verified.verified_at_ms, Some(130));

    let events = trust_events(&conn).expect("trust events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].action, "init");
    assert_eq!(events[1].action, "verify");
}

#[test]
fn trust_device_verify_rejects_fingerprint_mismatch() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let created = trust_device_init(&conn, "desktop", "tester", 120).expect("trust init");
    let err = trust_device_verify(
        &conn,
        &created.device_id,
        "aaaaaaaa:bbbbbbbb:cccccccc:dddddddd:eeeeeeee:ffffffff:11111111:22222222",
        "tester",
        130,
    )
    .expect_err("fingerprint mismatch should fail");
    assert_eq!(err.code, "KC_TRUST_FINGERPRINT_MISMATCH");
}

#[test]
fn trust_device_list_is_deterministic() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 100).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let first = trust_device_init(&conn, "a", "tester", 100).expect("trust init a");
    let second = trust_device_init(&conn, "b", "tester", 200).expect("trust init b");

    let listed = trust_device_list(&conn).expect("trust list");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].device_id, first.device_id);
    assert_eq!(listed[1].device_id, second.device_id);
}

#[test]
fn fingerprint_format_is_grouped_hex() {
    let pubkey = [0xABu8; 32];
    let fp = format_device_fingerprint(&pubkey);
    assert_eq!(fp.len(), 71);
    assert!(fp.chars().all(|c| c == ':' || c.is_ascii_hexdigit()));
    assert_eq!(fp.to_ascii_lowercase(), fp);
}
