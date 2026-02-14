# Tauri RPC Surface (v1)

## Purpose
Versioned RPC surface v1, envelope schema, and determinism notes.

## Invariants
- Thin RPC; versioned types; envelope ok/data or ok/error; errors are AppError v1.

## Acceptance Tests
- Round-trip serialization tests; UI types match; desktop gates pass.

         ## Envelope schema (v1)
         ```json
         {
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "kc://schemas/rpc-envelope/v1",
  "oneOf": [
    {
      "type": "object",
      "required": [
        "ok",
        "data"
      ],
      "properties": {
        "ok": {
          "const": true
        },
        "data": {}
      },
      "additionalProperties": false
    },
    {
      "type": "object",
      "required": [
        "ok",
        "error"
      ],
      "properties": {
        "ok": {
          "const": false
        },
        "error": {
          "$schema": "https://json-schema.org/draft/2020-12/schema",
          "$id": "kc://schemas/app-error/v1",
          "type": "object",
          "required": [
            "schema_version",
            "code",
            "category",
            "message",
            "retryable",
            "details"
          ],
          "properties": {
            "schema_version": {
              "const": 1
            },
            "code": {
              "type": "string"
            },
            "category": {
              "type": "string"
            },
            "message": {
              "type": "string"
            },
            "retryable": {
              "type": "boolean"
            },
            "details": {}
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    }
  ]
}
         ```

         ## Methods (v1)
         - vault_init, vault_open
         - vault_lock_status, vault_unlock, vault_lock
         - vault_encryption_status, vault_encryption_enable, vault_encryption_migrate
         - ingest_scan_folder, ingest_inbox_start/stop
         - search_query (includes now_ms param for deterministic tests)
         - locator_resolve
         - export_bundle, verify_bundle
         - ask_question
         - events_list, jobs_list
         - sync_status, sync_push, sync_pull
           - `sync_pull` accepts optional `auto_merge` with supported value `conservative`
         - sync_merge_preview
         - lineage_query
         - lineage_query_v2
         - lineage_overlay_add, lineage_overlay_remove, lineage_overlay_list

         ### Compatibility note
         - `lineage_query` (v1 response) remains supported during transition.
         - `lineage_query_v2` is the primary method for overlay-aware lineage responses.

         ## Determinism note
         - now_ms is passed by caller (UI/tests) to make snapshots deterministic.
