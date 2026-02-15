use jsonschema::JSONSchema;

fn draft_lineage_schema() -> serde_json::Value {
    serde_json::json!({
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "$id": "kc://schemas/draft/lineage-query/v1",
      "type": "object",
      "required": ["schema_version", "status", "activation_phase", "nodes", "edges"],
      "properties": {
        "schema_version": { "const": 1 },
        "status": { "const": "draft" },
        "activation_phase": { "const": "N3" },
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
fn schema_draft_lineage_accepts_representative_payload() {
    let schema =
        JSONSchema::compile(&draft_lineage_schema()).expect("compile draft lineage schema");
    let value = serde_json::json!({
      "schema_version": 1,
      "status": "draft",
      "activation_phase": "N3",
      "nodes": [
        { "node_id": "chunk:1", "kind": "chunk", "label": "Chunk 1", "metadata": {} },
        { "node_id": "doc:1", "kind": "doc", "label": "Doc 1", "metadata": {} }
      ],
      "edges": [
        { "from_node_id": "doc:1", "to_node_id": "chunk:1", "relation": "contains", "evidence": "draft" }
      ]
    });

    assert!(schema.is_valid(&value));
}

#[test]
fn schema_draft_lineage_rejects_missing_edges() {
    let schema =
        JSONSchema::compile(&draft_lineage_schema()).expect("compile draft lineage schema");
    let invalid = serde_json::json!({
      "schema_version": 1,
      "status": "draft",
      "activation_phase": "N3",
      "nodes": []
    });

    assert!(!schema.is_valid(&invalid));
}
