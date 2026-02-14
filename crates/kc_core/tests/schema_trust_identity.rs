use jsonschema::JSONSchema;
use kc_core::db::open_db;
use kc_core::trust::{trust_device_init, trust_device_verify};
use kc_core::trust_identity::{
    trust_device_enroll, trust_device_verify_chain, trust_identity_complete, trust_identity_start,
};
use kc_core::vault::vault_init;

fn identity_start_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/trust-identity-start/v1",
      "type": "object",
      "required": ["provider_id", "state", "authorization_url"],
      "properties": {
        "provider_id": { "type": "string", "minLength": 1 },
        "state": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "authorization_url": { "type": "string", "minLength": 1 }
      },
      "additionalProperties": false
    })
}

fn identity_session_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/trust-identity-session/v1",
      "type": "object",
      "required": [
        "session_id",
        "provider_id",
        "subject",
        "claim_subset_json",
        "issued_at_ms",
        "expires_at_ms",
        "created_at_ms"
      ],
      "properties": {
        "session_id": { "type": "string", "pattern": "^[0-9a-fA-F-]{36}$" },
        "provider_id": { "type": "string", "minLength": 1 },
        "subject": { "type": "string", "minLength": 1 },
        "claim_subset_json": { "type": "string", "minLength": 2 },
        "issued_at_ms": { "type": "integer" },
        "expires_at_ms": { "type": "integer" },
        "created_at_ms": { "type": "integer" }
      },
      "additionalProperties": false
    })
}

fn device_certificate_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/device-certificate/v1",
      "type": "object",
      "required": [
        "cert_id",
        "device_id",
        "provider_id",
        "subject",
        "cert_chain_hash",
        "issued_at_ms",
        "expires_at_ms",
        "verified_at_ms",
        "created_at_ms"
      ],
      "properties": {
        "cert_id": { "type": "string", "pattern": "^[0-9a-fA-F-]{36}$" },
        "device_id": { "type": "string", "pattern": "^[0-9a-fA-F-]{36}$" },
        "provider_id": { "type": "string", "minLength": 1 },
        "subject": { "type": "string", "minLength": 1 },
        "cert_chain_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "issued_at_ms": { "type": "integer" },
        "expires_at_ms": { "type": "integer" },
        "verified_at_ms": { "type": ["integer", "null"] },
        "created_at_ms": { "type": "integer" }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_trust_identity_accepts_valid_payloads() {
    let temp = tempfile::tempdir().expect("tempdir");
    let vault_path = temp.path().join("vault");
    vault_init(&vault_path, "demo", 1).expect("vault init");
    let conn = open_db(&vault_path.join("db/knowledge.sqlite")).expect("open db");

    let started = trust_identity_start(&conn, "default", 100).expect("identity start");
    let completed =
        trust_identity_complete(&conn, "default", "sub:alice@example.com", 101).expect("complete");

    let device = trust_device_init(&conn, "desktop", "tester", 102).expect("device init");
    let verified_device =
        trust_device_verify(&conn, &device.device_id, &device.fingerprint, "tester", 103)
            .expect("device verify");
    let enrolled =
        trust_device_enroll(&conn, "default", &verified_device.device_id, 104).expect("enroll");
    let verified_chain =
        trust_device_verify_chain(&conn, &verified_device.device_id, 105).expect("verify chain");

    let start_schema = JSONSchema::compile(&identity_start_schema()).expect("compile start schema");
    let session_schema =
        JSONSchema::compile(&identity_session_schema()).expect("compile session schema");
    let cert_schema =
        JSONSchema::compile(&device_certificate_schema()).expect("compile cert schema");

    assert!(start_schema.is_valid(&serde_json::to_value(started).expect("serialize start")));
    assert!(session_schema.is_valid(&serde_json::to_value(completed).expect("serialize session")));
    assert!(cert_schema.is_valid(&serde_json::to_value(enrolled).expect("serialize enrolled cert")));
    assert!(cert_schema
        .is_valid(&serde_json::to_value(verified_chain).expect("serialize verified cert")));
}

#[test]
fn schema_device_certificate_rejects_missing_chain_hash() {
    let schema = JSONSchema::compile(&device_certificate_schema()).expect("compile cert schema");
    let invalid = serde_json::json!({
      "cert_id": "11111111-1111-1111-1111-111111111111",
      "device_id": "22222222-2222-2222-2222-222222222222",
      "provider_id": "default",
      "subject": "alice@example.com",
      "issued_at_ms": 1,
      "expires_at_ms": 2,
      "verified_at_ms": null,
      "created_at_ms": 1
    });
    assert!(!schema.is_valid(&invalid));
}
