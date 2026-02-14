# Advanced Lineage UI v1

## Purpose
Provide a read-only lineage query and rendering contract where lineage assembly occurs in `kc_core`, RPC is thin pass-through, and UI renders server-provided order without re-ranking.

## Invariants
- UI has no lineage business logic and branches on `AppError.code` only.
- Tauri RPC only marshals request/response envelopes.
- Lineage ordering is deterministic:
  - nodes sorted by `kind`, then `node_id`
  - edges sorted by `from_node_id`, then `to_node_id`, then `relation`, then `evidence`
- Query is read-only; no lineage mutation APIs in v1.

## Acceptance Tests
- `kc_core` lineage tests verify deterministic ordering and failure codes.
- RPC schema tests verify `lineage_query` request strictness (`now_ms` required, unknown fields rejected).
- UI route and feature tests verify `lineage` screen uses typed RPC only.
- Desktop and Rust gates pass.

## Lineage Query Request (v1)
`LineageQueryReqV1`
- `vault_path: String`
- `seed_doc_id: String`
- `depth: i64` (`>= 1`)
- `now_ms: i64`

## Lineage Query Response (v1)
`LineageQueryResV1`
- `schema_version: i64` (const `1`)
- `seed_doc_id: String`
- `depth: i64`
- `generated_at_ms: i64`
- `nodes: Vec<LineageNodeV1>`
- `edges: Vec<LineageEdgeV1>`

`LineageNodeV1`
- `node_id: String`
- `kind: String` (for example: `doc`, `object`, `source`, `canonical`, `chunk`, `event`)
- `label: String`
- `metadata: JSON`

`LineageEdgeV1`
- `from_node_id: String`
- `to_node_id: String`
- `relation: String`
- `evidence: String`

## RPC Surface (v1)
- Method: `lineage_query`
- Envelope: `{ ok: true, data } | { ok: false, error }` per `spec/19-tauri-rpc-surface.md`.

## Determinism and Version Boundaries
- Response order is contract-level and must not vary across runs for identical DB state and request.
- `now_ms` is caller-provided for deterministic snapshots and tests.
- Any ordering change, relation semantic change, or request/response field semantic change requires version bump.

## Failure Modes and AppError Codes
- `KC_LINEAGE_INVALID_DEPTH`: `depth < 1`.
- `KC_LINEAGE_DOC_NOT_FOUND`: `seed_doc_id` is not present.
- `KC_LINEAGE_QUERY_FAILED`: storage/query/decode failure while assembling lineage.

## Stop Conditions
- Any lineage ranking/assembly logic appears in UI or Tauri.
- Response ordering drifts from deterministic contract.
- Schema registry or schema validation tests are not updated for contract changes.
