# Verifier and Reporting (v1)

## Purpose
Verifier behavior, stable exit codes, deterministic report ordering.

## Invariants
- Tier 1: exit codes stable; errors sorted deterministically.

## Acceptance Tests
- Verifier tests map failure modes to codes; report ordering stable.

         ## Report JSON schema (v1)
         ```json
         {
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "kc://schemas/verifier-report/v1",
  "type": "object",
  "required": [
    "report_version",
    "status",
    "exit_code",
    "errors",
    "checked"
  ],
  "properties": {
    "report_version": {
      "const": 1
    },
    "status": {
      "type": "string",
      "enum": [
        "ok",
        "failed"
      ]
    },
    "exit_code": {
      "type": "integer"
    },
    "errors": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [
          "code",
          "path"
        ],
        "properties": {
          "code": {
            "type": "string"
          },
          "path": {
            "type": "string"
          },
          "expected": {
            "type": [
              "string",
              "null"
            ]
          },
          "actual": {
            "type": [
              "string",
              "null"
            ]
          }
        },
        "additionalProperties": false
      }
    },
    "checked": {
      "type": "object",
      "required": [
        "objects",
        "indexes"
      ],
      "properties": {
        "objects": {
          "type": "integer"
        },
        "indexes": {
          "type": "integer"
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
         ```

         ## Exit codes (stable)
         - 0 OK
         - 20 MANIFEST_INVALID_JSON
         - 21 MANIFEST_SCHEMA_INVALID
           (includes deterministic ZIP metadata violations)
           (includes `RECOVERY_ESCROW_METADATA_MISMATCH`)
         - 31 DB_HASH_MISMATCH
           (also used for DB_ENCRYPTION_MISMATCH)
         - 40 OBJECT_MISSING
         - 41 OBJECT_HASH_MISMATCH
           (also used for OBJECT_ENCRYPTION_MISMATCH)
         - 60 INTERNAL_ERROR
         (Full list in this spec's narrative in earlier plan; must match implementation.)

         ## Ordering
         - errors sorted by code asc, path asc

         ## Error codes (AppError)
         - `KC_VERIFY_FAILED`
         - `KC_VERIFY_OBJECT_HASH_MISMATCH`
