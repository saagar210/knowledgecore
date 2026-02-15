use jsonschema::JSONSchema;

fn lineage_role_binding_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/lineage-role-binding/v2",
      "type": "object",
      "required": ["subject_id", "role_name", "role_rank", "granted_by", "granted_at_ms"],
      "properties": {
        "subject_id": { "type": "string", "minLength": 1 },
        "role_name": { "type": "string", "minLength": 1 },
        "role_rank": { "type": "integer" },
        "granted_by": { "type": "string", "minLength": 1 },
        "granted_at_ms": { "type": "integer" }
      },
      "additionalProperties": false
    })
}

fn lineage_scope_lock_status_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/lineage-lock-scope-status/v2",
      "type": "object",
      "required": [
        "scope_kind",
        "scope_value",
        "held",
        "owner",
        "acquired_at_ms",
        "expires_at_ms",
        "expired"
      ],
      "properties": {
        "scope_kind": { "type": "string", "enum": ["doc", "set"] },
        "scope_value": { "type": "string", "minLength": 1 },
        "held": { "type": "boolean" },
        "owner": { "type": ["string", "null"] },
        "acquired_at_ms": { "type": ["integer", "null"] },
        "expires_at_ms": { "type": ["integer", "null"] },
        "expired": { "type": "boolean" }
      },
      "additionalProperties": false
    })
}

fn lineage_policy_binding_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/lineage-policy-binding/v3",
      "type": "object",
      "required": [
        "subject_id",
        "policy_id",
        "policy_name",
        "effect",
        "priority",
        "condition_json",
        "bound_by",
        "bound_at_ms"
      ],
      "properties": {
        "subject_id": { "type": "string", "minLength": 1 },
        "policy_id": { "type": "string", "pattern": "^blake3:[0-9a-f]+$" },
        "policy_name": { "type": "string", "minLength": 1 },
        "effect": { "type": "string", "enum": ["allow", "deny"] },
        "priority": { "type": "integer" },
        "condition_json": { "type": "string", "minLength": 2 },
        "bound_by": { "type": "string", "minLength": 1 },
        "bound_at_ms": { "type": "integer" }
      },
      "additionalProperties": false
    })
}

fn lineage_policy_audit_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/lineage-policy-audit/v3",
      "type": "object",
      "required": [
        "ts_ms",
        "subject_id",
        "action",
        "doc_id",
        "allowed",
        "reason",
        "matched_policy_id",
        "details_json"
      ],
      "properties": {
        "ts_ms": { "type": "integer" },
        "subject_id": { "type": "string", "minLength": 1 },
        "action": { "type": "string", "minLength": 1 },
        "doc_id": { "type": ["string", "null"] },
        "allowed": { "type": "boolean" },
        "reason": {
          "type": "string",
          "enum": ["policy_allow", "policy_deny", "no_matching_allow_policy"]
        },
        "matched_policy_id": { "type": ["string", "null"] },
        "details_json": { "type": "string", "minLength": 2 }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_lineage_role_binding_accepts_valid_payload() {
    let schema =
        JSONSchema::compile(&lineage_role_binding_schema()).expect("compile role binding schema");
    let payload = serde_json::json!({
      "subject_id": "alice",
      "role_name": "editor",
      "role_rank": 20,
      "granted_by": "admin",
      "granted_at_ms": 1710000000000i64
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_lineage_role_binding_rejects_missing_rank() {
    let schema =
        JSONSchema::compile(&lineage_role_binding_schema()).expect("compile role binding schema");
    let payload = serde_json::json!({
      "subject_id": "alice",
      "role_name": "editor",
      "granted_by": "admin",
      "granted_at_ms": 1710000000000i64
    });
    assert!(!schema.is_valid(&payload));
}

#[test]
fn schema_lineage_scope_lock_status_accepts_valid_payload() {
    let schema = JSONSchema::compile(&lineage_scope_lock_status_schema())
        .expect("compile scope status schema");
    let payload = serde_json::json!({
      "scope_kind": "doc",
      "scope_value": "doc-1",
      "held": true,
      "owner": "alice",
      "acquired_at_ms": 1710000000000i64,
      "expires_at_ms": 1710000900000i64,
      "expired": false
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_lineage_scope_lock_status_rejects_unknown_scope_kind() {
    let schema = JSONSchema::compile(&lineage_scope_lock_status_schema())
        .expect("compile scope status schema");
    let payload = serde_json::json!({
      "scope_kind": "global",
      "scope_value": "doc-1",
      "held": true,
      "owner": "alice",
      "acquired_at_ms": 1710000000000i64,
      "expires_at_ms": 1710000900000i64,
      "expired": false
    });
    assert!(!schema.is_valid(&payload));
}

#[test]
fn schema_lineage_policy_binding_accepts_valid_payload() {
    let schema = JSONSchema::compile(&lineage_policy_binding_schema())
        .expect("compile lineage policy binding schema");
    let payload = serde_json::json!({
      "subject_id": "alice",
      "policy_id": "blake3:1234abcd",
      "policy_name": "allow-overlay",
      "effect": "allow",
      "priority": 200,
      "condition_json": "{\"action\":\"lineage.overlay.write\"}",
      "bound_by": "desktop",
      "bound_at_ms": 1710000000000i64
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_lineage_policy_binding_rejects_unknown_effect() {
    let schema = JSONSchema::compile(&lineage_policy_binding_schema())
        .expect("compile lineage policy binding schema");
    let payload = serde_json::json!({
      "subject_id": "alice",
      "policy_id": "blake3:1234abcd",
      "policy_name": "allow-overlay",
      "effect": "maybe",
      "priority": 200,
      "condition_json": "{\"action\":\"lineage.overlay.write\"}",
      "bound_by": "desktop",
      "bound_at_ms": 1710000000000i64
    });
    assert!(!schema.is_valid(&payload));
}

#[test]
fn schema_lineage_policy_audit_accepts_valid_payload() {
    let schema = JSONSchema::compile(&lineage_policy_audit_schema())
        .expect("compile lineage policy audit schema");
    let payload = serde_json::json!({
      "ts_ms": 1710000000000i64,
      "subject_id": "alice",
      "action": "lineage.overlay.write",
      "doc_id": "doc:123",
      "allowed": false,
      "reason": "policy_deny",
      "matched_policy_id": "blake3:abcd1234",
      "details_json": "{\"action\":\"lineage.overlay.write\",\"doc_id\":\"doc:123\",\"matched_effect\":\"deny\",\"matched_policy_id\":\"blake3:abcd1234\",\"matched_policy_name\":\"deny-all\",\"reason\":\"policy_deny\",\"subject_id\":\"alice\"}"
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_lineage_policy_audit_rejects_unknown_reason() {
    let schema = JSONSchema::compile(&lineage_policy_audit_schema())
        .expect("compile lineage policy audit schema");
    let payload = serde_json::json!({
      "ts_ms": 1710000000000i64,
      "subject_id": "alice",
      "action": "lineage.overlay.write",
      "doc_id": "doc:123",
      "allowed": false,
      "reason": "unknown",
      "matched_policy_id": null,
      "details_json": "{}"
    });
    assert!(!schema.is_valid(&payload));
}
