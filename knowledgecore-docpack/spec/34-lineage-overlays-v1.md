# Lineage Overlays v1

## Purpose
Define writable lineage overlays while preserving immutable system lineage graph semantics.

## Invariants
- System provenance edges/nodes are immutable.
- User edits are represented as overlay entries only.
- Merge of system + overlays is deterministic and versioned.
- UI renders server order and does not re-rank/rewrite graph semantics.

## Non-goals
- Direct mutation of system lineage rows.
- Collaborative/real-time editing.
- Automatic lineage inference from UI actions.

## Interface contracts
- Core overlay operations:
  - `lineage_overlay_add`
  - `lineage_overlay_remove`
  - `lineage_overlay_list`
  - `lineage_query_v2`
- Overlay entry schema:
  - `overlay_id: String`
  - `doc_id: String`
  - `from_node_id: String`
  - `to_node_id: String`
  - `relation: String`
  - `evidence: String`
  - `created_at_ms: i64`
  - `created_by: String`

## Determinism and version-boundary rules
- Query merge order:
  - nodes: `kind`, then `node_id`
  - edges: `from_node_id`, `to_node_id`, `relation`, `evidence`
  - overlay edges sort using same key and are included in merged list deterministically.
- Any change to merge precedence or overlay schema requires version boundary review.

## Failure modes and AppError mapping
- `KC_LINEAGE_OVERLAY_INVALID`: invalid overlay payload.
- `KC_LINEAGE_OVERLAY_NOT_FOUND`: remove/list reference missing overlay.
- `KC_LINEAGE_OVERLAY_CONFLICT`: duplicate overlay identity conflict.
- Existing lineage query errors remain valid for base graph failures.

## Acceptance tests
- Overlay add/remove/list CLI and core tests pass.
- `lineage_query_v2` deterministic ordering remains stable.
- Base lineage remains unchanged after overlay operations.
- RPC/UI tests confirm no client-side lineage business logic.

## Rollout gate and stop conditions
### Rollout gate
- Q1/Q2 milestones pass canonical Rust + desktop + RPC/schema gates.

### Stop conditions
- Any direct mutation of base lineage state by overlay APIs.
- Non-deterministic merged output ordering.
