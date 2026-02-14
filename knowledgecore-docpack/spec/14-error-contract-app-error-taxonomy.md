# AppError Contract and Taxonomy (v1)

## Purpose
AppError schema v1 and stable error taxonomy; UI branches on code only.

## Invariants
- UI branches on AppError.code only; message is not contract.
- Codes never reused.
- details is JSON-serializable.

## Acceptance Tests
- Serialization round-trips; UI tests branch on code only; all referenced codes present.

## AppError JSON schema (v1)
```json
{
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
```

## Retryability guidance (v1)
- IO transient: retryable=true
- DB migration/integrity: retryable=false
- Schema validation: retryable=false
- Tool unavailable (PDFium/Tesseract): retryable=true after install/config
- Ask provider unavailable: retryable=true

## Taxonomy (v1)
- Vault/DB: `KC_VAULT_*`, `KC_DB_*`
- Hash/Canon JSON: `KC_HASH_*`, `KC_CANON_JSON_*`
- Ingest: `KC_INGEST_*`, `KC_INBOX_*`, `KC_TIMESTAMP_*`
- Extract: `KC_CANONICAL_*`, `KC_PDFIUM_UNAVAILABLE`, `KC_TESSERACT_UNAVAILABLE`, `KC_OCR_FAILED`
- Chunking: `KC_CHUNK_*`
- Index: `KC_FTS_*`, `KC_VECTOR_*`, `KC_EMBEDDING_*`
- Retrieval: `KC_RETRIEVAL_*`
- Locator/Snippet: `KC_LOCATOR_*`, `KC_SNIPPET_*`
- Export/Verify: `KC_EXPORT_*`, `KC_VERIFY_*`
- Ask/Trace: `KC_ASK_*`, `KC_TRACE_*`
- RPC/Internal: `KC_RPC_*`, `KC_INTERNAL_ERROR`
- Encryption:
  - `KC_ENCRYPTION_KEY_INVALID`
  - `KC_ENCRYPTION_UNSUPPORTED`
  - `KC_ENCRYPTION_REQUIRED`
  - `KC_ENCRYPTION_MIGRATION_FAILED`
- Sync:
  - `KC_SYNC_TARGET_INVALID`
  - `KC_SYNC_STATE_FAILED`
  - `KC_SYNC_CONFLICT`
  - `KC_SYNC_APPLY_FAILED`

## RPC mapping
- RPC responses return either `{ ok: true, data }` or `{ ok: false, error: AppError }` without changing `error.code`.
