# Cross-Device Sync v1 (Design Lock)

## Purpose
Define Phase N2 contracts for local-first snapshot sync with explicit conflict artifacts and deterministic logs.

## Invariants
- Sync remains deferred in Phase L with no active runtime behavior.
- Local vault remains source of truth; no remote dependency is required to run the application.
- Any sync decision that affects Tier 1 objects/manifests must be deterministic and versioned.

## Non-goals
- No real-time collaborative editing.
- No peer-to-peer transport protocol implementation.
- No background daemon sync loop in Phase L.

## Interface Contracts (Draft)
### `SyncManifestDraftV1`
- `schema_version: i64` (const `1`)
- `status: String` (const `"draft"`)
- `activation_phase: String` (const `"N2"`)
- `vault_id: String`
- `snapshot_id: String`
- `created_at_ms: i64`
- `objects_hash: String`
- `db_hash: String`
- `conflicts: Vec<SyncConflictDraftV1>`

### `SyncConflictDraftV1`
- `path: String`
- `local_hash: String`
- `remote_hash: String`
- `resolution_strategy: String` (planned default `"emit_conflict_artifact"`)

### `SyncOperationReqDraftV1`
- `vault_path: String`
- `target: String`
- `mode: String` (`"pull"` or `"push"`)
- `now_ms: i64`

### Draft RPC and CLI shell contracts
- Preview RPC method (feature-gated): `preview_sync_status`
- Preview CLI shell (feature-gated): `kc_cli preview capability --name sync`

## Determinism and Version-Boundary Rules
- Snapshot IDs must be derived deterministically from canonicalized sync inputs.
- Conflict lists must be sorted deterministically by `path`, then hash values.
- Replay logs must retain stable ordering with explicit sequence IDs.
- Any conflict policy change requires schema boundary note and deterministic fixture updates.

## Failure Modes and AppError Code Map (Draft)
- `KC_DRAFT_SYNC_NOT_IMPLEMENTED`: preview shell reached; sync behavior intentionally absent.
- `KC_DRAFT_SYNC_CONFLICT_POLICY_UNIMPLEMENTED`: sync conflict resolution path invoked before activation phase.
- `KC_DRAFT_PREVIEW_UNKNOWN_CAPABILITY`: preview shell requested unsupported capability name.

## Acceptance Tests (Phase L)
- Schema validation test for `SyncManifestDraftV1` exists and passes.
- Preview scaffold exposes deterministic sync draft metadata and placeholder errors.
- Default build has no active sync command or RPC behavior.
- Existing ingest/export/verify behaviors remain unchanged.

## Rollout Gate and Stop Conditions
### Rollout gate to start Phase N2
- Conflict policy and audit requirements approved.
- Snapshot replay invariants approved.
- Registry draft entry promoted to active implementation entry.

### Stop conditions
- Any active sync mutation behavior lands in Phase L.
- Draft contracts lack schema validation coverage.
