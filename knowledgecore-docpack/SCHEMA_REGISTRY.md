# SCHEMA_REGISTRY.md

## Purpose
Authoritative registry of all versioned schemas and contracts. Any schema addition/change must update this file and include validation tests.

## Invariants
- Every schema has: name, version, canonical path, producer(s), consumer(s), invariants, compatibility rules, bump rules.
- Tier 1 schemas define ordering and hashing rules explicitly.
- RPC boundary types are schemas and must be versioned.

## Acceptance Tests
- Schema validation tests exist and run in CI for each schema category.
- Registry stays consistent with `spec/*` and `apps/desktop` types.

## Registry Table

| Schema | Ver | Path | Producer | Consumer | Tier | Invariants | Compat Rules | Bump Rules |
|---|---:|---|---|---|---|---|---|---|
| Canonical JSON | 1 | `spec/00-canonical-json.md` | kc_core | all | 1 | stable canonical JSON bytes | N/A | bump on encoding change |
| vault.json | 1 | `spec/02-vault-topology-and-vault-json.md` | kc_core | all | 1-adj | UUID vault_id; schema_version=1 | additive optional ok | bump on breaking change |
| SQLite schema | 1 | `spec/03-sqlite-schema-v1-and-migrations.md` | kc_core | all | 1 | migrations deterministic | additive ok | bump user_version on change |
| Locator v1 | 1 | `spec/10-locator-v1-and-resolver.md` | kc_core | all | 1 | [start,end) char indices; strict hash check | additive hints ok | bump on semantics change |
| Export manifest | 1 | `spec/12-export-bundles-and-manifest.md` | kc_core | verifier/UI | 1 | deterministic ordering; db hash | additive blocks ok | bump on ordering change |
| Verifier report | 1 | `spec/13-verifier-and-reporting.md` | kc_cli | UI/automation | 1 | stable exit codes; deterministic ordering | additive ok | bump on exit/order change |
| AppError | 1 | `spec/14-error-contract-app-error-taxonomy.md` | all | UI/CLI/RPC | 1-adj | UI branches on code only | additive codes ok | bump on struct change |
| Trace log | 1 | `spec/17-trace-log-schema-and-redaction.md` | kc_ask | UI/automation | 1 | deterministic array ordering | additive ok | bump on struct change |
| RPC envelope | 1 | `spec/19-tauri-rpc-surface.md` | src-tauri | UI | 1-adj | ok/data or ok/error | additive methods ok | bump on breaking change |

## Schema validation workflow
- JSON schema validation tests (Rust `jsonschema` crate):
  - `cargo test -p kc_core -- schema_*`
  - `cargo test -p kc_cli -- schema_*`
- RPC round-trip serialization tests:
  - `cargo test -p apps_desktop_tauri -- rpc_*` (crate name assumption; finalized in implementation)

## Assumption
- Formal JSON Schemas are embedded in spec files and mirrored into Rust tests as literals to validate at build time.
