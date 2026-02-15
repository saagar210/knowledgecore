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
    "encryption",
    "db_encryption",
    "recovery_escrow",
    "packaging",
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
    "encryption": {
      "type": "object",
      "required": [
        "enabled",
        "mode",
        "kdf"
      ],
      "properties": {
        "enabled": { "type": "boolean" },
        "mode": { "type": "string" },
        "key_reference": { "type": ["string", "null"] },
        "kdf": {
          "type": "object",
          "required": ["algorithm"],
          "properties": {
            "algorithm": { "type": "string" }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    },
    "db_encryption": {
      "type": "object",
      "required": [
        "enabled",
        "mode",
        "kdf"
      ],
      "properties": {
        "enabled": { "type": "boolean" },
        "mode": { "type": "string" },
        "key_reference": { "type": ["string", "null"] },
        "kdf": {
          "type": "object",
          "required": ["algorithm", "memory_kib", "iterations", "parallelism", "salt_id"],
          "properties": {
            "algorithm": { "type": "string" },
            "memory_kib": { "type": "integer", "minimum": 1 },
            "iterations": { "type": "integer", "minimum": 1 },
            "parallelism": { "type": "integer", "minimum": 1 },
            "salt_id": { "type": "string" }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    },
    "recovery_escrow": {
      "type": "object",
      "required": ["enabled", "provider", "updated_at_ms", "descriptor"],
      "properties": {
        "enabled": { "type": "boolean" },
        "provider": { "type": "string", "minLength": 1 },
        "updated_at_ms": { "type": ["integer", "null"] },
        "descriptor": {
          "type": ["object", "null"]
        }
      },
      "additionalProperties": false
    },
    "toolchain_registry": {
      "type": "object"
    },
    "packaging": {
      "type": "object",
      "required": ["format", "zip_policy"],
      "properties": {
        "format": { "type": "string", "enum": ["folder", "zip"] },
        "zip_policy": {
          "type": "object",
          "required": ["compression", "mtime", "file_mode"],
          "properties": {
            "compression": { "type": "string", "const": "stored" },
            "mtime": { "type": "string", "const": "1980-01-01T00:00:00Z" },
            "file_mode": { "type": "string", "const": "0644" }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
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
          "storage_hash",
          "encrypted",
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
          "storage_hash": {
            "type": "string",
            "pattern": "^blake3:[0-9a-f]{64}$"
          },
          "encrypted": {
            "type": "boolean"
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
         - object storage_hash/encrypted derived from copied payload bytes
         - vectors: relative_path asc
         - recovery_escrow invariant: `enabled=false` requires `provider=none`, `updated_at_ms=null`, and `descriptor=null`; `enabled=true` requires provider not `none`, integer `updated_at_ms`, and object `descriptor`
         - when present, `providers[]` and `escrow_descriptors[]` are ordered by provider priority (`aws`,`gcp`,`azure`,`hsm`,`local`,`private_kms`) with lexical tie-breaks, and provider ids must come from this supported set

         ## Error codes
         - `KC_EXPORT_FAILED`
         - `KC_EXPORT_MANIFEST_WRITE_FAILED`
