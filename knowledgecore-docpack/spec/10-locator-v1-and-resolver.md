# Locator v1 and Resolver

## Purpose
Locator schema and strict resolver contract.

## Invariants
- Tier 1: [start,end) char indices into canonical text; canonical_hash required in strict contexts; hints non-authoritative.

## Acceptance Tests
- Locator schema validation tests and strict resolver substring tests pass.

         ## JSON schema (v1)
         ```json
         {
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "kc://schemas/locator/v1",
  "type": "object",
  "required": [
    "v",
    "doc_id",
    "canonical_hash",
    "range"
  ],
  "properties": {
    "v": {
      "const": 1
    },
    "doc_id": {
      "type": "string",
      "pattern": "^blake3:[0-9a-f]{64}$"
    },
    "canonical_hash": {
      "type": "string",
      "pattern": "^blake3:[0-9a-f]{64}$"
    },
    "range": {
      "type": "object",
      "required": [
        "start",
        "end"
      ],
      "properties": {
        "start": {
          "type": "integer",
          "minimum": 0
        },
        "end": {
          "type": "integer",
          "minimum": 0
        }
      },
      "additionalProperties": false
    },
    "hints": {
      "type": "object",
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "pdf",
            "html",
            "md",
            "text"
          ]
        },
        "pages": {
          "type": "object",
          "required": [
            "start",
            "end"
          ],
          "properties": {
            "start": {
              "type": "integer",
              "minimum": 1
            },
            "end": {
              "type": "integer",
              "minimum": 1
            }
          },
          "additionalProperties": false
        },
        "heading_path": {
          "type": "array",
          "items": {
            "type": "string"
          }
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
         ```

         ## Strict resolver
         - compare canonical_hash; validate range; return exact substring.

         ## Error codes
         - `KC_LOCATOR_INVALID_SCHEMA`
         - `KC_LOCATOR_CANONICAL_HASH_MISMATCH`
         - `KC_LOCATOR_RANGE_OOB`
