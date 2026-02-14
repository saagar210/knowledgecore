# Lineage Collaborative Turn Lock v1

## Purpose
Define deterministic, turn-based edit-lock contracts for lineage overlay mutations.

## Invariants
- Lock scope is per `doc_id`.
- Overlay writes are rejected without a valid active lock token.
- Lease timeout is fixed at 15 minutes and evaluated with caller-provided `now_ms`.
- Lock acquisition, status, and mutation checks run in core only.
- UI/Tauri do not arbitrate lock ownership.

## Non-goals
- Distributed consensus locking.
- Background lock renewal or heartbeat daemons.
- Multi-document transactional lock orchestration.

## Interface contracts
- Core lock operations:
  - `lineage_lock_acquire(conn, doc_id, owner, now_ms)`
  - `lineage_lock_release(conn, doc_id, token)`
  - `lineage_lock_status(conn, doc_id, now_ms)`
- Overlay mutation contracts:
  - `lineage_overlay_add(..., lock_token, created_at_ms, ...)`
  - `lineage_overlay_remove(..., lock_token, now_ms)`
- Surface contracts:
  - CLI lock workflows: `kc_cli lineage lock acquire|release|status`
  - Tauri RPC lock workflows: `lineage_lock_acquire|release|status`
  - Tauri overlay RPCs require `lock_token`; remove also requires `now_ms`
- Lock schema fields:
  - `doc_id`
  - `owner`
  - `token`
  - `acquired_at_ms`
  - `expires_at_ms`

## Determinism and version-boundary rules
- Lock token derivation is deterministic for fixed inputs.
- Lock status ordering is deterministic by `doc_id`.
- Expiration checks use caller `now_ms` so test replay remains deterministic.
- Any change to token derivation, lease duration, or mutation enforcement semantics requires version-boundary review.

## Failure modes and AppError mapping
- `KC_LINEAGE_LOCK_HELD`: lock acquisition attempted while active lock exists.
- `KC_LINEAGE_LOCK_INVALID`: missing/unknown lock token.
- `KC_LINEAGE_LOCK_EXPIRED`: provided token belongs to expired lease.
- Existing lineage query/overlay error codes remain valid.

## Acceptance tests
- Acquire/status/release round-trip is deterministic.
- Competing holder is rejected with `KC_LINEAGE_LOCK_HELD`.
- Overlay add/remove without valid token fails with `KC_LINEAGE_LOCK_INVALID`.
- Expired lock token fails with `KC_LINEAGE_LOCK_EXPIRED`.
- Lock schema validation tests pass.

## Rollout gate and stop conditions
### Rollout gate
- `cargo test -p kc_core -- lineage`
- `cargo test -p kc_core -- schema_`
- Canonical Rust gate from `knowledgecore-docpack/AGENTS.md`.

### Stop conditions
- Any overlay mutation path succeeds without lock validation.
- Any non-deterministic lock lease evaluation under fixed `now_ms`.
- Missing schema registry updates or missing lock schema tests.
