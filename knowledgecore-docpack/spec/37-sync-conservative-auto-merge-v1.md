# Sync Conservative Auto-Merge v2

## Purpose
Define deterministic conservative auto-merge contracts for sync pull flows. Auto-merge is opt-in and only permitted when local and remote changes are provably disjoint.

## Invariants
- Merge decisions are computed in `kc_core` only.
- UI and Tauri must never compute merge safety independently.
- Conservative auto-merge is allowed only when:
  - object change sets have no overlapping `object_hash` values, and
  - lineage overlay change sets have no overlapping `overlay_id` values.
- `conservative_plus_v2` extends conservative checks with:
  - no trust-chain mismatch, and
  - no active lineage lock conflict.
- Overlap must hard-fail with deterministic `AppError.code`.
- Merge preview reports are deterministic in ordering and schema.

## Non-goals
- Heuristic merge conflict resolution.
- Automatic destructive overwrite.
- Semantic merge beyond deterministic disjoint-set checks.

## Interface contracts
- Core types:
  - `SyncMergeChangeSetV1`
  - `SyncMergePreviewReportV1`
  - `SyncMergeContextV2`
  - `SyncMergePreviewReportV2`
- Core functions:
  - `merge_preview_conservative(local, remote, now_ms) -> SyncMergePreviewReportV1`
  - `ensure_conservative_merge_safe(report) -> Result<(), AppError>`
  - `merge_preview_with_policy_v2(local, remote, ctx, policy, now_ms) -> SyncMergePreviewReportV2`
  - `ensure_conservative_plus_v2_merge_safe(report) -> Result<(), AppError>`
- CLI surface:
  - `kc_cli sync merge-preview <vault_path> <target_uri> --policy <conservative|conservative_plus_v2> --now-ms <ms>`
  - `kc_cli sync pull <vault_path> <target_uri> --auto-merge <conservative|conservative_plus_v2> --now-ms <ms>`
- RPC surface:
  - `sync_merge_preview` with optional `policy`

## Determinism and version-boundary rules
- Change-set normalization rules:
  - hash arrays are validated, deduplicated, and lexicographically sorted
  - overlay id arrays are deduplicated and lexicographically sorted
- Overlap arrays and `reasons` are sorted lexicographically.
- `decision_trace` entries are deterministic, fixed-order strings for equivalent inputs.
- `generated_at_ms` is caller-supplied and required to keep replayability deterministic.
- Any change to overlap semantics, normalization, or report ordering requires version-boundary review.

## Failure modes and AppError mapping
- `KC_SYNC_MERGE_NOT_SAFE`: conservative merge rejected because overlap exists.
- `KC_SYNC_MERGE_PRECONDITION_FAILED`: invalid merge preview input (invalid object hash or invalid overlay id).
- `KC_SYNC_MERGE_POLICY_UNSUPPORTED`: requested policy is unknown/unsupported.
- `KC_SYNC_MERGE_TRUST_CONFLICT`: policy rejected due to trust chain mismatch.
- `KC_SYNC_MERGE_LOCK_CONFLICT`: policy rejected due to active lineage lock conflict.
- Existing sync conflict/lock/auth codes remain authoritative for transport-level failures.

## Acceptance tests
- Preview normalizes and deduplicates unsorted input deterministically.
- Overlap detection for object hashes and overlay ids is deterministic.
- Unsafe preview triggers `KC_SYNC_MERGE_NOT_SAFE`.
- Invalid preview input triggers `KC_SYNC_MERGE_PRECONDITION_FAILED`.
- Schema validation tests pass for preview report payloads.

## Rollout gate and stop conditions
### Rollout gate
- `cargo test -p kc_core -- sync_merge`
- `cargo test -p kc_core -- schema_`
- Canonical Rust gate from `knowledgecore-docpack/AGENTS.md`.

### Stop conditions
- Any merge-safe result when overlaps exist.
- Any non-deterministic ordering in preview output.
- Missing schema registry updates or missing schema validation tests.
