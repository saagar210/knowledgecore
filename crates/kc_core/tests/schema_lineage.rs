use jsonschema::JSONSchema;

fn lineage_response_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/lineage-query/v1",
      "type": "object",
      "required": [
        "schema_version",
        "seed_doc_id",
        "depth",
        "generated_at_ms",
        "nodes",
        "edges"
      ],
      "properties": {
        "schema_version": { "const": 1 },
        "seed_doc_id": { "type": "string" },
        "depth": { "type": "integer", "minimum": 1 },
        "generated_at_ms": { "type": "integer" },
        "nodes": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["node_id", "kind", "label", "metadata"],
            "properties": {
              "node_id": { "type": "string" },
              "kind": { "type": "string" },
              "label": { "type": "string" },
              "metadata": {}
            },
            "additionalProperties": false
          }
        },
        "edges": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["from_node_id", "to_node_id", "relation", "evidence"],
            "properties": {
              "from_node_id": { "type": "string" },
              "to_node_id": { "type": "string" },
              "relation": { "type": "string" },
              "evidence": { "type": "string" }
            },
            "additionalProperties": false
          }
        }
      },
      "additionalProperties": false
    })
}

#[test]
fn schema_lineage_query_accepts_valid_payload() {
    let schema = JSONSchema::compile(&lineage_response_schema()).expect("compile lineage schema");
    let payload = serde_json::json!({
      "schema_version": 1,
      "seed_doc_id": "doc-1",
      "depth": 2,
      "generated_at_ms": 100,
      "nodes": [
        {
          "node_id": "doc:doc-1",
          "kind": "doc",
          "label": "doc-1",
          "metadata": { "doc_id": "doc-1" }
        }
      ],
      "edges": [
        {
          "from_node_id": "doc:doc-1",
          "to_node_id": "object:abc",
          "relation": "originates_from",
          "evidence": "docs.original_object_hash"
        }
      ]
    });

    assert!(schema.is_valid(&payload));
}

#[test]
fn schema_lineage_query_rejects_missing_nodes() {
    let schema = JSONSchema::compile(&lineage_response_schema()).expect("compile lineage schema");
    let payload = serde_json::json!({
      "schema_version": 1,
      "seed_doc_id": "doc-1",
      "depth": 1,
      "generated_at_ms": 100,
      "edges": []
    });
    assert!(!schema.is_valid(&payload));
}
