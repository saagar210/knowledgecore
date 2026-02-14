# Export Bundles and Manifest (v1)

## Purpose
Deterministic export folder bundle and manifest schema + ordering rules.

## Invariants
- Tier 1: deterministic manifest bytes; ordered lists sorted as specified; db hash included.

## Acceptance Tests
- Export then verify passes; manifest ordering snapshot stable.

         ## Manifest JSON schema (v1)
         ```json
         {
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "kc://schemas/export-manifest/v1",
  "type": "object",
  "required": [
    "manifest_version",
    "vault_id",
    "schema_versions",
    "chunking_config_hash",
    "db",
    "objects"
  ],
  "properties": {
    "manifest_version": {
      "const": 1
    },
    "vault_id": {
      "type": "string",
      "format": "uuid"
    },
    "schema_versions": {
      "type": "object"
    },
    "toolchain_registry": {
      "type": "object"
    },
    "chunking_config_hash": {
      "type": "string",
      "pattern": "^blake3:[0-9a-f]{64}$"
    },
    "embedding": {
      "type": "object"
    },
    "db": {
      "type": "object",
      "required": [
        "relative_path",
        "hash"
      ],
      "properties": {
        "relative_path": {
          "type": "string"
        },
        "hash": {
          "type": "string",
          "pattern": "^blake3:[0-9a-f]{64}$"
        }
      },
      "additionalProperties": false
    },
    "objects": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [
          "relative_path",
          "hash",
          "bytes"
        ],
        "properties": {
          "relative_path": {
            "type": "string"
          },
          "hash": {
            "type": "string",
            "pattern": "^blake3:[0-9a-f]{64}$"
          },
          "bytes": {
            "type": "integer",
            "minimum": 0
          }
        },
        "additionalProperties": false
      }
    },
    "indexes": {
      "type": "object"
    }
  },
  "additionalProperties": false
}
         ```

         ## Ordering (Tier 1)
         - objects: hash asc then relative_path asc
         - vectors: relative_path asc

         ## Error codes
         - `KC_EXPORT_FAILED`
         - `KC_EXPORT_MANIFEST_WRITE_FAILED`
