# Deferred Capability Schema Drafts (Phase L)

## Purpose
Define draft schema surfaces for deferred capabilities and the promotion rules from draft to active runtime schemas.

## Invariants
- Draft schemas are non-runtime in Phase L.
- Draft schemas require validation tests even before activation.
- Promotion to active runtime schema requires explicit phase assignment and acceptance gate sign-off.

## Non-goals
- No runtime schema activation in Phase L.
- No migration execution or production data transformation.

## Interface Contracts (Draft Registry)
### `EncryptionMetadataDraftV1`
- Source spec: `spec/22-encryption-at-rest-v1-design-lock.md`
- Activation phase: `M`
- Status: `draft`

### `ZipPackagingMetadataDraftV1`
- Source spec: `spec/23-deterministic-zip-packaging-v1-design-lock.md`
- Activation phase: `N1`
- Status: `draft`

### `SyncManifestDraftV1`
- Source spec: `spec/24-cross-device-sync-v1-design-lock.md`
- Activation phase: `N2`
- Status: `draft`

### `LineageQueryResDraftV1`
- Source spec: `spec/25-advanced-lineage-ui-v1-design-lock.md`
- Activation phase: `N3`
- Status: `draft`

## Determinism and Version-Boundary Rules
- Draft schema payload examples must serialize deterministically for test fixtures.
- Any field addition/removal in draft schema requires:
  - schema validation test update,
  - schema registry draft row update,
  - change note in the owning design-lock spec.
- Promotion from draft to active requires compatibility and bump-rule declaration in `SCHEMA_REGISTRY.md`.

## Failure Modes and AppError Code Map (Draft)
- `KC_DRAFT_SCHEMA_NOT_ACTIVE`: runtime path attempted to consume draft schema directly.
- `KC_DRAFT_SCHEMA_VALIDATION_FAILED`: draft payload does not match draft schema contract.
- `KC_DRAFT_PREVIEW_UNKNOWN_CAPABILITY`: preview shell requested unsupported capability name.

## Acceptance Tests (Phase L)
- `kc_core` draft schema tests validate representative payloads for all four draft schema families.
- CLI/Tauri preview tests prove default builds remain non-exposing.
- Canonical gates from `AGENTS.md` remain green.

## Rollout Gate and Stop Conditions
### Rollout gate for draft promotion
- Owning phase (`M`/`N1`/`N2`/`N3`) is approved.
- Draft schema row is moved/replicated into active schema section with final bump rules.
- Migration and verifier strategy is approved where applicable.

### Stop conditions
- Draft schema used as active runtime contract without promotion.
- Schema registry and validation tests fall out of sync.
