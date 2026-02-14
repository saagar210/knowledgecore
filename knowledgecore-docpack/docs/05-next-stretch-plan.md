# Next Stretch Plan (Post D–K)

## Title
KnowledgeCore Next Stretch: Deferred Capabilities Promotion (Phases L–N)

## Summary
This plan defines the first post-D–K execution horizon as three candidate phases that promote previously deferred capabilities in controlled increments. Each phase is decision-complete with clear goals, non-goals, invariants, interfaces, tests, acceptance criteria, and risks.

## Global Invariants
- No business logic in UI; UI branches on `AppError.code` only.
- Tauri remains thin RPC only.
- Tier 1 determinism guarantees must not regress.
- Schema changes require `SCHEMA_REGISTRY.md` update and validation tests.
- Stop/go loop applies to every milestone: fail, diagnose, fix, rerun to green.

## Phase L — Deferred Feature Readiness and Design Lock

### Goal
Prepare deferred capabilities for implementation by producing ratified specs, schema drafts, and migration strategy without shipping runtime behavior changes.

### Non-goals
- No encryption runtime enablement.
- No ZIP export format rollout.
- No sync engine implementation.
- No advanced lineage UI implementation.

### Deliverables
- Readiness specs for each deferred capability:
  - encryption at rest design draft,
  - deterministic ZIP packaging contract draft,
  - sync model and conflict policy draft,
  - lineage UX contract draft.
- Explicit capability gates, fallback/disable behavior, and AppError code maps.
- Schema draft table with planned version bumps and compatibility notes.

### API / Schema impact expectations
- No production API changes in L.
- Draft-only schema additions may be documented but not activated.

### Tests / Acceptance criteria
- Documentation lint passes (if enabled locally).
- Existing canonical gates remain green.
- Each deferred capability has a signed-off spec section with:
  - invariants,
  - failure modes,
  - rollout gate.

### Risks and mitigations
- Risk: over-design without implementation pressure.
  - Mitigation: lock only decisions necessary for Phase M/N execution.
- Risk: schema draft drift.
  - Mitigation: add explicit draft status markers and owners.

## Phase M — Security and Encryption-at-Rest Implementation

### Goal
Introduce optional encryption-at-rest for vault objects and/or database with explicit versioning and migration paths.

### Non-goals
- Cross-device key escrow/sync.
- Multi-tenant remote KMS orchestration.

### Proposed architecture decisions
- Encryption scope v1:
  - object store payloads first,
  - SQLite file encryption optional second (if toolchain support is stable).
- Key source:
  - local passphrase-derived key (PBKDF/Argon2),
  - no external secrets service required in v1.
- Vault manifest marker:
  - add encryption metadata block and key-derivation parameters.

### API / Schema changes (expected)
- `vault.json` v2 or additive v1 extension (decision at kickoff):
  - encryption enabled flag,
  - kdf params,
  - cipher suite identifier.
- Export manifest extension:
  - encryption metadata for verifier.
- AppError additions:
  - `KC_ENCRYPTION_KEY_INVALID`,
  - `KC_ENCRYPTION_UNSUPPORTED`,
  - `KC_ENCRYPTION_MIGRATION_FAILED`.

### Migration / compatibility
- Existing plaintext vaults remain supported.
- Opt-in migration command creates encrypted replicas with verification.
- Verifier distinguishes encrypted vs plaintext expectations.

### Tests / Acceptance criteria
- Deterministic crypto metadata serialization tests.
- Migration round-trip tests (plaintext -> encrypted -> readable).
- Verifier tests for encrypted bundle checks.
- Full gate suite green.

### Risks and mitigations
- Risk: performance regressions in ingest/query.
  - Mitigation: benchmark comparisons with threshold policy.
- Risk: key-loss operational failure.
  - Mitigation: explicit backup/recovery instructions and hard warnings.

## Phase N — Packaging, Sync, and Lineage Expansion

### Goal
Promote deterministic ZIP packaging and introduce sync + lineage v1 features in bounded scope.

### Non-goals
- Real-time collaborative editing.
- Full graph analytics lineage engine.

### Subphase N1: Deterministic ZIP packaging
- Produce zip bundles from deterministic folder export.
- Stable file order, timestamps, permissions policy.
- Verifier validates ZIP determinism metadata.

### Subphase N2: Cross-device sync v1
- Pull/push snapshot sync model with conflict strategy:
  - last-write with explicit conflict artifact generation.
- Local-first operation remains default.
- Explicit sync log and replay safety checks.

### Subphase N3: Advanced lineage UI v1
- Read-only lineage graph over existing provenance/locator data.
- No inference/business logic in UI; all lineage computation via RPC/core.

### API / Schema changes (expected)
- Export manifest extension for ZIP packaging metadata.
- Sync manifest schema v1.
- RPC method additions for sync operations and lineage queries.
- UI route additions for sync/lineage screens.

### Tests / Acceptance criteria
- ZIP determinism snapshot tests.
- Sync conflict and replay safety tests.
- Lineage UI RPC contract tests and smoke tests.
- Full Rust/UI gates and bench smoke green.

### Risks and mitigations
- Risk: determinism drift in ZIP format across platforms.
  - Mitigation: pinned zip writer/tooling + golden fixtures per platform.
- Risk: sync conflict complexity.
  - Mitigation: begin with coarse snapshot sync and explicit conflict files.
- Risk: lineage UI leaks business logic.
  - Mitigation: boundary tests and checklist enforcement.

## Cross-Phase Acceptance Gates
- Rust gate:
  - `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli`
- Desktop gate:
  - `pnpm lint && pnpm test && pnpm tauri build`
- Bench smoke gate:
  - `cargo run -p kc_cli -- bench run --corpus v1`

## Assumptions and Defaults
- D–K baseline remains stable and merged.
- `master` is single active branch baseline before L starts.
- Deferred items stay deferred until phase kickoff approval.
- Schema versions stay v1 unless breaking changes force bumps.

## Suggested Sequencing
1. Execute Phase L as a documentation/spec lock sprint.
2. Execute Phase M milestone M1 for object encryption.
3. Execute Phase M milestone M2 for migration and verifier hardening.
4. Execute Phase N milestone N1 for deterministic ZIP packaging.
5. Execute Phase N milestone N2 for sync v1.
6. Execute Phase N milestone N3 for lineage UI v1.
