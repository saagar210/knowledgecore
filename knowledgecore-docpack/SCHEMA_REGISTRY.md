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
| vault.json | 2 | `spec/27-encryption-at-rest-v1.md` | kc_core | all | 1-adj | UUID vault_id; schema_version=2 for new vaults; v1 normalized to v2 in memory | read v1 + v2 during migration window | bump on breaking change |
| encryption metadata (object store) | 1 | `spec/27-encryption-at-rest-v1.md` | kc_core | kc_cli/src-tauri/ui settings | 1-adj | mode `object_store_xchacha20poly1305`; deterministic nonce derivation context; plaintext hash remains canonical | additive optional fields ok | bump on mode/kdf semantics change |
| SQLite schema | 1 | `spec/03-sqlite-schema-v1-and-migrations.md` | kc_core | all | 1 | migrations deterministic | additive ok | bump user_version on change |
| Locator v1 | 1 | `spec/10-locator-v1-and-resolver.md` | kc_core | all | 1 | [start,end) char indices; strict hash check | additive hints ok | bump on semantics change |
| Export manifest | 1 | `spec/12-export-bundles-and-manifest.md` | kc_core | verifier/UI | 1 | deterministic ordering; `vault_id` UUID; db hash; chunking_config_hash uses canonical config hashing; required `encryption` block; object entries carry `hash` (plaintext hash), `storage_hash` (stored payload hash), and `encrypted` flag | additive blocks ok | bump on ordering/hash rule change |
| ZIP packaging metadata | 1 | `spec/28-deterministic-zip-packaging-v1.md` | kc_core | kc_cli verifier | 1 | entry order lexical; compression stored; fixed mtime `1980-01-01T00:00:00Z`; file mode `0644` | additive fields ok | bump on any deterministic ZIP policy change |
| Sync snapshots/head/conflict | 2 | `spec/31-sync-s3-transport-v1.md` + `spec/33-cross-device-passphrase-trust-v1.md` | kc_core | kc_cli/src-tauri/ui | 1 | deterministic snapshot id derivation `kc.sync.snapshot.v2`; deterministic head/conflict serialization; trust metadata required for schema_version=2 heads; no auto-merge | additive optional fields ok when schema_version preserved | bump on snapshot id, trust model, lock protocol, or conflict semantics change |
| Sync target URI | 2 | `spec/31-sync-s3-transport-v1.md` | kc_core | kc_cli/src-tauri/ui | 1-adj | supports `file://`, plain path, and `s3://bucket/prefix`; deterministic canonical target display | additive optional schemes need explicit review | bump on parse semantics or supported scheme changes |
| Lineage query | 1 | `spec/30-advanced-lineage-ui-v1.md` | kc_core | src-tauri/ui | 1 | deterministic nodes (`kind`,`node_id`) and edges (`from`,`to`,`relation`,`evidence`) ordering; read-only query contract | additive optional metadata fields ok | bump on request/response semantics or ordering rule change |
| Verifier report | 1 | `spec/13-verifier-and-reporting.md` | kc_cli | UI/automation | 1 | stable exit codes (0/20/21/31/40/41/60); deterministic ordering; schema-validated manifest input; encryption-state mismatches map into code 41 | additive ok | bump on exit/order/schema rule change |
| AppError | 1 | `spec/14-error-contract-app-error-taxonomy.md` | all | UI/CLI/RPC | 1-adj | UI branches on code only | additive codes ok | bump on struct change |
| Trace log | 1 | `spec/17-trace-log-schema-and-redaction.md` | kc_ask | UI/automation | 1 | `trace_id`/`vault_id` UUID; deterministic retrieval chunk ordering + locator ordering | additive ok | bump on struct change |
| RPC envelope | 1 | `spec/19-tauri-rpc-surface.md` | src-tauri | UI | 1-adj | strict one-of envelope; methods include `ingest_inbox_start/stop` and `vault_encryption_status/enable/migrate`; deterministic reqs carry `now_ms` | additive methods ok | bump on breaking change |

## Draft Schemas (Phase L, non-runtime)

- No remaining draft schemas. Phase L draft lineage contract was promoted to active in N3 (`spec/30-advanced-lineage-ui-v1.md`).

## Schema validation workflow
- JSON schema validation tests (Rust `jsonschema` crate):
  - `cargo test -p kc_core -- schema_*`
  - `cargo test -p kc_cli -- schema_*`
- Draft schema validation tests (Phase L scaffolding):
  - `cargo test -p kc_core -- schema_draft_*`
- RPC round-trip serialization tests:
  - `cargo test -p apps_desktop_tauri -- rpc_*`
  - Deterministic RPC request schema tests:
    - `cargo test -p apps_desktop_tauri -- rpc_schema_*`

## Assumption
- Formal JSON Schemas are embedded in spec files and mirrored into Rust tests as literals to validate at build time.
