# Advanced Lineage UI v1 (Design Lock)

## Purpose
Define Phase N3 contracts for read-only lineage visualization built from existing provenance and locator data.

## Invariants
- Phase L introduces no lineage runtime feature behavior.
- Lineage computation remains in core/RPC boundaries; UI does not add truth-layer logic.
- UI continues to branch on `AppError.code` only.

## Non-goals
- No graph inference engine.
- No editable lineage graph.
- No speculative ranking or business-rule interpretation in UI.

## Interface Contracts (Draft)
### `LineageNodeDraftV1`
- `node_id: String`
- `kind: String` (examples: `doc`, `chunk`, `event`, `export`)
- `label: String`
- `metadata: serde_json::Value`

### `LineageEdgeDraftV1`
- `from_node_id: String`
- `to_node_id: String`
- `relation: String`
- `evidence: String`

### `LineageQueryReqDraftV1`
- `vault_path: String`
- `seed_doc_id: String`
- `depth: i64`
- `now_ms: i64`

### `LineageQueryResDraftV1`
- `schema_version: i64` (const `1`)
- `status: String` (const `"draft"`)
- `activation_phase: String` (const `"N3"`)
- `nodes: Vec<LineageNodeDraftV1>`
- `edges: Vec<LineageEdgeDraftV1>`

### Draft RPC and CLI shell contracts
- Preview RPC method (feature-gated): `preview_lineage_status`
- Preview CLI shell (feature-gated): `kc_cli preview capability --name lineage`

## Determinism and Version-Boundary Rules
- Lineage response ordering must be deterministic:
  - nodes sorted by `kind`, then `node_id`
  - edges sorted by `from_node_id`, then `to_node_id`, then `relation`
- Any lineage query parameter semantics change requires a version boundary update.
- UI rendering order must follow RPC order and never re-rank client-side.

## Failure Modes and AppError Code Map (Draft)
- `KC_DRAFT_LINEAGE_NOT_IMPLEMENTED`: preview shell reached; lineage behavior intentionally absent.
- `KC_DRAFT_LINEAGE_QUERY_UNIMPLEMENTED`: lineage query invoked before activation phase.
- `KC_DRAFT_PREVIEW_UNKNOWN_CAPABILITY`: preview shell requested unsupported capability name.

## Acceptance Tests (Phase L)
- Schema validation test for lineage draft response exists and passes.
- UI and Tauri preview stubs remain behind compile/runtime preview gates.
- Default desktop build exposes no lineage preview method by default.
- Existing desktop tests remain green.

## Rollout Gate and Stop Conditions
### Rollout gate to start Phase N3
- RPC contract review approved.
- UI boundary checklist includes lineage-specific checks.
- Registry draft entry promoted to active implementation entry.

### Stop conditions
- Any lineage business logic appears in UI.
- Any runtime lineage command is enabled in default builds during Phase L.
