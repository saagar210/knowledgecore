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
