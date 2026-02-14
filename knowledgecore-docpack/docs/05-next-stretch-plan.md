# Next Stretch Plan (Post-N3)

## Title
KnowledgeCore Remaining Roadmap: Phases O, P, Q, R

## Summary
This execution horizon closes the remaining deferred scope after N3:
- O: sync transport v2 (URI targets + S3-compatible adapter + trust metadata)
- P: SQLCipher DB encryption with vault schema v3 and unlock/session surfaces
- Q: lineage overlay write/edit model (overlay-only, immutable system lineage)
- R: preview scaffold retirement and final hardening

## Global Invariants
- Local-first remains default; remote sync is optional.
- UI has no business logic and branches on `AppError.code` only.
- Tauri layer remains thin RPC orchestration only.
- Tier 1 determinism and ordering contracts must not regress.
- Schema changes must update `SCHEMA_REGISTRY.md` and schema validation tests.
- Milestones are merge-as-we-go with stop/fix/rerun gates.

## Phase O — Sync Transport v2 (Sync-first)

### Goal
Generalize sync target handling from local filesystem path semantics to URI semantics while preserving existing local sync behavior.

### Scope
- Transport abstraction and URI parser (`file://`, plain path, `s3://`).
- S3-compatible adapter with deterministic snapshot/head/conflict object layout.
- Passphrase trust metadata in sync head/manifests and compatibility checks.
- CLI/Tauri/UI sync surfaces updated to accept URI targets.

### Non-goals
- Auto-merge conflict resolution.
- Device enrollment PKI.
- Background daemonized sync loop.

### Acceptance
- Existing local path sync still works unchanged.
- `s3://` sync push/pull/status supported through CLI and desktop settings.
- Deterministic conflict artifacts and stable error codes.

## Phase P — SQLCipher DB Encryption v1

### Goal
Add SQLCipher-backed DB encryption as active at-rest protection for the SQLite file while keeping v1/v2 vault compatibility.

### Scope
- Vault schema v3 with DB encryption metadata.
- `vault_open` support for v1/v2/v3 normalized model.
- DB key/unlock contract for CLI and RPC.
- Migration flow with rollback-safe strategy and integrity checks.
- Export/verifier schema and checks aligned with encrypted DB metadata.

### Non-goals
- External KMS.
- Per-record encryption overlays.
- Removing object-store encryption support.

### Acceptance
- Plaintext and encrypted vaults are both openable through supported flows.
- Wrong key/fingerprint fails deterministically with stable `AppError.code`.
- Post-migration plaintext DB artifacts are not left in active vault paths.

## Phase Q — Lineage Overlay Workflows v1

### Goal
Enable user-editable lineage via overlays only, preserving immutable system lineage.

### Scope
- Overlay storage table + migration.
- Overlay CRUD in core and CLI.
- `lineage_query_v2` merged response with deterministic ordering rules.
- RPC and UI support for overlay add/remove/list and v2 query rendering.

### Non-goals
- Direct mutation of system lineage edges.
- Automatic inference/ranking in UI.
- Collaborative lineage editing.

### Acceptance
- Overlay changes never mutate base lineage graph.
- Deterministic merged ordering remains stable across repeated runs.
- UI renders response order without client-side reordering.

## Phase R — Runtime Surface Cleanup + Final Hardening

### Goal
Remove obsolete Phase L preview scaffolding and finalize docs/risk closure for O–Q.

### Scope
- Remove `phase_l_preview` runtime surfaces and preview-only tests.
- Keep only active runtime contracts.
- Final readiness/follow-up docs update and full gate run.

### Acceptance
- No preview-only runtime path remains.
- Full Rust + desktop + RPC/schema + bench smoke gates pass on `master`.

## Canonical Gates
- Rust: `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli`
- Desktop: `pnpm lint && pnpm test && pnpm tauri build`
- Bench smoke: `cargo run -p kc_cli -- bench run --corpus v1` (twice at final consolidation)
