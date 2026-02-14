use jsonschema::JSONSchema;

fn sync_merge_preview_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/sync-merge-preview/v1",
      "type": "object",
      "required": [
        "schema_version",
        "merge_policy",
        "safe",
        "generated_at_ms",
        "local",
        "remote",
        "overlap",
        "reasons"
      ],
      "properties": {
        "schema_version": { "const": 1 },
        "merge_policy": { "type": "string", "const": "conservative_v1" },
        "safe": { "type": "boolean" },
        "generated_at_ms": { "type": "integer" },
        "local": { "$ref": "#/$defs/change_set" },
        "remote": { "$ref": "#/$defs/change_set" },
        "overlap": { "$ref": "#/$defs/change_set" },
        "reasons": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": ["object_hash_overlap", "lineage_overlay_overlap"]
          }
        }
      },
      "$defs": {
        "change_set": {
          "type": "object",
          "required": ["object_hashes", "lineage_overlay_ids"],
          "properties": {
            "object_hashes": {
              "type": "array",
              "items": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" }
            },
            "lineage_overlay_ids": {
              "type": "array",
              "items": { "type": "string", "minLength": 1 }
            }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    })
}

fn sync_merge_preview_schema_v2() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/sync-merge-preview/v2",
      "type": "object",
      "required": [
        "schema_version",
        "merge_policy",
        "safe",
        "generated_at_ms",
        "local",
        "remote",
        "overlap",
        "reasons",
        "decision_trace"
      ],
      "properties": {
        "schema_version": { "const": 2 },
        "merge_policy": { "type": "string", "const": "conservative_plus_v2" },
        "safe": { "type": "boolean" },
        "generated_at_ms": { "type": "integer" },
        "local": { "$ref": "#/$defs/change_set" },
        "remote": { "$ref": "#/$defs/change_set" },
        "overlap": { "$ref": "#/$defs/change_set" },
        "reasons": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": [
              "object_hash_overlap",
              "lineage_overlay_overlap",
              "trust_chain_mismatch",
              "lineage_lock_conflict"
            ]
          }
        },
        "decision_trace": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        }
      },
      "$defs": {
        "change_set": {
          "type": "object",
          "required": ["object_hashes", "lineage_overlay_ids"],
          "properties": {
            "object_hashes": {
              "type": "array",
              "items": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" }
            },
            "lineage_overlay_ids": {
              "type": "array",
              "items": { "type": "string", "minLength": 1 }
            }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_sync_merge_preview_accepts_valid_payload() {
    let schema =
        JSONSchema::compile(&sync_merge_preview_schema()).expect("compile sync merge schema");
    let payload = serde_json::json!({
      "schema_version": 1,
      "merge_policy": "conservative_v1",
      "safe": false,
      "generated_at_ms": 123,
      "local": {
        "object_hashes": [
          "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ],
        "lineage_overlay_ids": ["overlay-a"]
      },
      "remote": {
        "object_hashes": [
          "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        ],
        "lineage_overlay_ids": ["overlay-a", "overlay-b"]
      },
      "overlap": {
        "object_hashes": [],
        "lineage_overlay_ids": ["overlay-a"]
      },
      "reasons": ["lineage_overlay_overlap"]
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_sync_merge_preview_rejects_unknown_reason() {
    let schema =
        JSONSchema::compile(&sync_merge_preview_schema()).expect("compile sync merge schema");
    let invalid = serde_json::json!({
      "schema_version": 1,
      "merge_policy": "conservative_v1",
      "safe": true,
      "generated_at_ms": 123,
      "local": { "object_hashes": [], "lineage_overlay_ids": [] },
      "remote": { "object_hashes": [], "lineage_overlay_ids": [] },
      "overlap": { "object_hashes": [], "lineage_overlay_ids": [] },
      "reasons": ["unknown_reason"]
    });
    assert!(!schema.is_valid(&invalid));
}

#[test]
fn schema_sync_merge_preview_v2_accepts_valid_payload() {
    let schema =
        JSONSchema::compile(&sync_merge_preview_schema_v2()).expect("compile sync merge v2 schema");
    let payload = serde_json::json!({
      "schema_version": 2,
      "merge_policy": "conservative_plus_v2",
      "safe": false,
      "generated_at_ms": 123,
      "local": {
        "object_hashes": [
          "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ],
        "lineage_overlay_ids": ["overlay-a"]
      },
      "remote": {
        "object_hashes": [
          "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ],
        "lineage_overlay_ids": ["overlay-b"]
      },
      "overlap": {
        "object_hashes": [
          "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ],
        "lineage_overlay_ids": []
      },
      "reasons": ["object_hash_overlap"],
      "decision_trace": [
        "policy=conservative_plus_v2",
        "overlap.object_hashes=1"
      ]
    });
    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_sync_merge_preview_v2_rejects_unknown_reason() {
    let schema =
        JSONSchema::compile(&sync_merge_preview_schema_v2()).expect("compile sync merge v2 schema");
    let invalid = serde_json::json!({
      "schema_version": 2,
      "merge_policy": "conservative_plus_v2",
      "safe": true,
      "generated_at_ms": 123,
      "local": { "object_hashes": [], "lineage_overlay_ids": [] },
      "remote": { "object_hashes": [], "lineage_overlay_ids": [] },
      "overlap": { "object_hashes": [], "lineage_overlay_ids": [] },
      "reasons": ["unknown_reason"],
      "decision_trace": ["policy=conservative_plus_v2"]
    });
    assert!(!schema.is_valid(&invalid));
}
