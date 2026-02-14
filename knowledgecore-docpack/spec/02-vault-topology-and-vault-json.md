# Vault Topology and vault.json (v1)

## Purpose
Defines vault folder layout and the `vault.json` schema v1.

## Invariants
- Vault is a user-visible folder (default root `~/KnowledgeVaults/<slug>/`).
- Multi-vault supported; default UX uses one active vault.
- `vault_id` is a UUID generated at init; does not affect doc identities.

## Acceptance Tests
- `kc_cli vault init` creates structure and writes schema-valid vault.json.
- `kc_core vault_open` validates schema_version and required fields.

         ## Folder structure (conceptual)
         - `vault.json`
         - `db/knowledge.sqlite`
         - `store/objects/<first2>/<object_hash>`
         - `index/vectors/` (LanceDB)
         - `Inbox/` and `Inbox/processed/`

         ## vault.json JSON schema (v1)
         ```json
         {
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "kc://schemas/vault.json/v1",
  "type": "object",
  "required": [
    "schema_version",
    "vault_id",
    "vault_slug",
    "created_at_ms",
    "db",
    "defaults",
    "toolchain"
  ],
  "properties": {
    "schema_version": {
      "const": 1
    },
    "vault_id": {
      "type": "string",
      "format": "uuid"
    },
    "vault_slug": {
      "type": "string",
      "minLength": 1
    },
    "created_at_ms": {
      "type": "integer"
    },
    "db": {
      "type": "object",
      "required": [
        "relative_path"
      ],
      "properties": {
        "relative_path": {
          "type": "string"
        }
      },
      "additionalProperties": false
    },
    "defaults": {
      "type": "object",
      "required": [
        "chunking_config_id",
        "embedding_model_id",
        "recency"
      ],
      "properties": {
        "chunking_config_id": {
          "type": "string"
        },
        "embedding_model_id": {
          "type": "string"
        },
        "recency": {
          "type": "object",
          "required": [
            "enabled"
          ],
          "properties": {
            "enabled": {
              "type": "boolean"
            }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    },
    "toolchain": {
      "type": "object",
      "required": [
        "pdfium",
        "tesseract"
      ],
      "properties": {
        "pdfium": {
          "type": "object",
          "required": [
            "identity"
          ],
          "properties": {
            "identity": {
              "type": "string"
            }
          },
          "additionalProperties": false
        },
        "tesseract": {
          "type": "object",
          "required": [
            "identity"
          ],
          "properties": {
            "identity": {
              "type": "string"
            }
          },
          "additionalProperties": false
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
         ```

         ## Version boundary behavior
         - Breaking change bumps schema_version.

         ## Error codes
         - `KC_VAULT_JSON_MISSING`
         - `KC_VAULT_JSON_INVALID`
         - `KC_VAULT_JSON_UNSUPPORTED_VERSION`
         - `KC_VAULT_INIT_FAILED`
