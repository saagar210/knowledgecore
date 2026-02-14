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
| vault.json | 3 | `spec/32-sqlite-encryption-sqlcipher-v1.md` | kc_core | all | 1-adj | UUID vault_id; schema_version=3 for new vaults; v1/v2 normalized to v3 in memory; includes `db_encryption` block | read v1 + v2 + v3 during migration window | bump on breaking change |
| encryption metadata (object store) | 1 | `spec/27-encryption-at-rest-v1.md` | kc_core | kc_cli/src-tauri/ui settings | 1-adj | mode `object_store_xchacha20poly1305`; deterministic nonce derivation context; plaintext hash remains canonical | additive optional fields ok | bump on mode/kdf semantics change |
| SQLite schema | 1 | `spec/03-sqlite-schema-v1-and-migrations.md` | kc_core | all | 1 | migrations deterministic | additive ok | bump user_version on change |
| Locator v1 | 1 | `spec/10-locator-v1-and-resolver.md` | kc_core | all | 1 | [start,end) char indices; strict hash check | additive hints ok | bump on semantics change |
| Export manifest | 1 | `spec/12-export-bundles-and-manifest.md` | kc_core | verifier/UI | 1 | deterministic ordering; `vault_id` UUID; db hash; chunking_config_hash uses canonical config hashing; required `encryption` and `db_encryption` blocks; object entries carry `hash` (plaintext hash), `storage_hash` (stored payload hash), and `encrypted` flag | additive blocks ok | bump on ordering/hash rule change |
| ZIP packaging metadata | 1 | `spec/28-deterministic-zip-packaging-v1.md` | kc_core | kc_cli verifier | 1 | entry order lexical; compression stored; fixed mtime `1980-01-01T00:00:00Z`; file mode `0644` | additive fields ok | bump on any deterministic ZIP policy change |
| Sync snapshots/head/conflict | 3 | `spec/31-sync-s3-transport-v1.md` + `spec/33-cross-device-passphrase-trust-v1.md` + `spec/35-device-trust-manual-verify-v1.md` + `spec/40-sync-head-signature-chain-v3.md` | kc_core | kc_cli/src-tauri/ui | 1 | deterministic snapshot id derivation `kc.sync.snapshot.v2`; deterministic head/conflict serialization; trust metadata required for schema_version>=2 heads; schema_version=3 heads require author identity chain fields (`author_device_id`, `author_fingerprint`, `author_signature`, `author_cert_id`, `author_chain_hash`) with deterministic signature payload hashing; no auto-merge | read v2 + v3 during migration window; emit v3 on new S3 head writes | bump on snapshot id, trust model, signature payload, lock protocol, or conflict semantics change |
| Sync merge preview report | 1 | `spec/37-sync-conservative-auto-merge-v1.md` | kc_core | kc_cli/src-tauri/ui | 1 | deterministic normalization/sorting for change-set arrays; deterministic overlap/reasons ordering; conservative policy only allows disjoint object hashes + disjoint lineage overlay ids | additive optional fields ok if ordering and overlap semantics unchanged | bump on merge policy semantics, ordering rules, or overlap logic |
| Sync target URI | 2 | `spec/31-sync-s3-transport-v1.md` | kc_core | kc_cli/src-tauri/ui | 1-adj | supports `file://`, plain path, and `s3://bucket/prefix`; deterministic canonical target display | additive optional schemes need explicit review | bump on parse semantics or supported scheme changes |
| Device trust manifest | 1 | `spec/35-device-trust-manual-verify-v1.md` | kc_core | kc_cli/src-tauri/ui | 1-adj | ed25519 device identity; deterministic fingerprint formatting and trust event ordering; unverified devices cannot author accepted remote heads | additive optional metadata fields ok | bump on fingerprint/signature payload semantics |
| Recovery bundle manifest | 1 | `spec/36-local-recovery-kit-v1.md` | kc_core | kc_cli/src-tauri/ui | 1-adj | deterministic `recovery_manifest.json` canonical bytes with checksum + payload hash; local-only output policy | additive metadata ok | bump on checksum/payload derivation semantics |
| Lineage query | 2 | `spec/34-lineage-overlays-v1.md` | kc_core | kc_cli/src-tauri/ui | 1 | deterministic nodes (`kind`,`node_id`) and edges (`from`,`to`,`relation`,`evidence`,`origin`) ordering; v2 merges immutable system edges with overlay edges deterministically | v1 read-only response remains supported during transition | bump on request/response semantics or ordering rule change |
| Lineage overlay entry | 1 | `spec/34-lineage-overlays-v1.md` | kc_core | kc_cli/src-tauri/ui | 1-adj | overlay_id deterministic hash identity; uniqueness on `(doc_id,from,to,relation,evidence)`; immutable system lineage untouched | additive optional metadata fields ok | bump on identity or ordering semantics change |
| Lineage lock lease/status | 1 | `spec/38-lineage-collab-turn-lock-v1.md` | kc_core | kc_cli/src-tauri/ui | 1-adj | per-doc turn lock with fixed 15-minute lease; deterministic lock token derivation and status serialization; overlay mutations require active matching token | additive optional metadata fields ok | bump on token derivation, lease semantics, or enforcement behavior |
| Verifier report | 1 | `spec/13-verifier-and-reporting.md` | kc_cli | UI/automation | 1 | stable exit codes (0/20/21/31/40/41/60); deterministic ordering; schema-validated manifest input; object encryption-state mismatches map into code 41 and DB encryption-state mismatches map into code 31 | additive ok | bump on exit/order/schema rule change |
| AppError | 1 | `spec/14-error-contract-app-error-taxonomy.md` | all | UI/CLI/RPC | 1-adj | UI branches on code only | additive codes ok | bump on struct change |
| Trace log | 1 | `spec/17-trace-log-schema-and-redaction.md` | kc_ask | UI/automation | 1 | `trace_id`/`vault_id` UUID; deterministic retrieval chunk ordering + locator ordering | additive ok | bump on struct change |
| RPC envelope | 1 | `spec/19-tauri-rpc-surface.md` | src-tauri | UI | 1-adj | strict one-of envelope; methods include lock-session RPC (`vault_lock_status`, `vault_unlock`, `vault_lock`), `ingest_inbox_start/stop`, `vault_encryption_status/enable/migrate`, `vault_recovery_status/generate/verify`, sync merge preview RPC (`sync_merge_preview`) and conservative pull option (`sync_pull.auto_merge`), lineage v2 + overlay RPCs (`lineage_query_v2`, `lineage_overlay_add/remove/list`) and lineage lock RPCs (`lineage_lock_acquire/release/status`); deterministic reqs carry caller-controlled timestamps (`now_ms`/`created_at_ms`) | additive methods ok | bump on breaking change |

## Draft Schemas (Phase L, non-runtime)

- Phase L draft runtime scaffolding was retired in R1 after O/P/Q activation.
- Design-lock specs `22`–`26` remain archival references and are not runtime contracts.

## Schema validation workflow
- JSON schema validation tests (Rust `jsonschema` crate):
  - `cargo test -p kc_core -- schema_*`
  - `cargo test -p kc_cli -- schema_*`
- RPC round-trip serialization tests:
  - `cargo test -p apps_desktop_tauri -- rpc_*`
  - Deterministic RPC request schema tests:
    - `cargo test -p apps_desktop_tauri -- rpc_schema_*`

## Assumption
- Formal JSON Schemas are embedded in spec files and mirrored into Rust tests as literals to validate at build time.
