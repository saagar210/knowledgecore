# Trace Log Schema and Redaction (v1)

## Purpose
Trace log schema v1 for local audit; includes redaction controls.

## Invariants
- Tier 1: deterministic ordering rules; local-only storage.
- Redaction preserves IDs and locators.

## Acceptance Tests
- Schema validation tests pass; redaction tests pass.

         ## JSON schema (v1)
         ```json
         {
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "kc://schemas/trace-log/v1",
  "type": "object",
  "required": [
    "schema_version",
    "trace_id",
    "ts_ms",
    "vault_id",
    "question",
    "retrieval",
    "model",
    "answer",
    "redaction"
  ],
  "properties": {
    "schema_version": {
      "const": 1
    },
    "trace_id": {
      "type": "string",
      "format": "uuid"
    },
    "ts_ms": {
      "type": "integer"
    },
    "vault_id": {
      "type": "string",
      "format": "uuid"
    },
    "question": {
      "type": "string"
    },
    "retrieval": {
      "type": "object"
    },
    "model": {
      "type": "object"
    },
    "answer": {
      "type": "object"
    },
    "redaction": {
      "type": "object"
    }
  },
  "additionalProperties": false
}
         ```

         ## Ordering rules
         - retrieval chunks in final order
         - locators sorted by doc_id/start/end

         ## Error codes
         - `KC_TRACE_WRITE_FAILED`
         - `KC_TRACE_REDACTION_FAILED`
