use jsonschema::validator_for;
use kc_core::trust::{trust_device_init, trust_device_verify};
use kc_core::{db::open_db, vault::vault_init};

fn trusted_device_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/trusted-device/v1",
      "type": "object",
      "required": [
        "device_id",
        "label",
        "pubkey",
        "fingerprint",
        "verified_at_ms",
        "created_at_ms"
      ],
      "properties": {
        "device_id": { "type": "string", "minLength": 1 },
        "label": { "type": "string", "minLength": 1 },
        "pubkey": { "type": "string", "pattern": "^[0-9a-f]{64}$" },
        "fingerprint": { "type": "string", "pattern": "^[0-9a-f]{8}(:[0-9a-f]{8}){7}$" },
        "verified_at_ms": { "type": ["integer", "null"] },
        "created_at_ms": { "type": "integer" }
      },
      "additionalProperties": false
    })
}

fn trust_event_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/trust-event/v1",
      "type": "object",
      "required": ["event_id", "device_id", "action", "actor", "ts_ms", "details_json"],
      "properties": {
        "event_id": { "type": "integer" },
        "device_id": { "type": "string", "minLength": 1 },
        "action": { "type": "string", "enum": ["init", "verify"] },
        "actor": { "type": "string", "minLength": 1 },
        "ts_ms": { "type": "integer" },
        "details_json": { "type": "string", "minLength": 2 }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_trusted_device_accepts_init_and_verify_payloads() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 1).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let init = trust_device_init(&conn, "laptop", "tester", 100).expect("init trust device");
    let verify = trust_device_verify(&conn, &init.device_id, &init.fingerprint, "tester", 101)
        .expect("verify trust device");

    let schema = validator_for(&trusted_device_schema()).expect("compile trusted_device schema");
    assert!(schema.is_valid(&serde_json::to_value(init).expect("serialize init")));
    assert!(schema.is_valid(&serde_json::to_value(verify).expect("serialize verify")));
}

#[test]
fn schema_trust_event_rejects_missing_details_json() {
    let schema = validator_for(&trust_event_schema()).expect("compile trust_event schema");
    let invalid = serde_json::json!({
      "event_id": 1,
      "device_id": "abc",
      "action": "verify",
      "actor": "tester",
      "ts_ms": 100
    });
    assert!(!schema.is_valid(&invalid));
}
