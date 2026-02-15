# Lineage Governance Conditions v3

## Purpose
Define deterministic condition-policy layering for lineage governance so overlay mutations require lock validity, RBAC allow, and policy-condition allow with deterministic audit trails.

## Invariants
- Overlay mutations remain core-only logic.
- Overlay mutation requires all of:
  - valid active lock token
  - RBAC allow for `lineage.overlay.write`
  - policy-condition allow decision for the request scope
- Policy evaluation is deterministic:
  - deny-default
  - explicit deny overrides allow
  - tie-break order is `priority ASC`, `policy_id ASC`, `subject_id ASC`
- Every policy decision writes deterministic audit records with canonical JSON details.
- UI and Tauri do not evaluate policy logic or precedence.

## Non-goals
- Replacing v2 role-rank RBAC precedence.
- Distributed/global policy replication.
- Client-side policy conflict resolution.

## Interface contracts
- Core policy APIs:
  - `lineage_policy_add(conn, policy_name, effect, condition_json, created_by, now_ms)`
  - `lineage_policy_bind(conn, subject_id, policy_name, bound_by, now_ms)`
  - `lineage_policy_list(conn)`
  - `lineage_policy_decision(conn, subject_id, action, doc_id)`
  - `ensure_lineage_policy_allows(conn, subject_id, action, doc_id, now_ms)`
- Core condition fields (`condition_json`):
  - `action` (optional string)
  - `doc_id_prefix` (optional string)
- Surface contracts:
  - CLI lineage policy flows: `kc_cli lineage policy add|bind|list`
  - RPC lineage policy flows: `lineage_policy_add|bind|list`
- DB schema additions:
  - `lineage_policies`
  - `lineage_policy_bindings`
  - `lineage_policy_audit`

## Determinism and version-boundary rules
- `condition_json` is canonical JSON before persistence.
- Policy IDs are deterministic for policy names:
  - `blake3("kc.lineage.policy.v3\\n" + policy_name)`
- Policy list ordering is deterministic by:
  - `priority ASC`, `policy_id ASC`, `subject_id ASC`
- Decision reasons are stable:
  - `policy_allow`, `policy_deny`, `no_matching_allow_policy`
- Audit ordering is deterministic by:
  - `ts_ms ASC`, `audit_id ASC`
- Any change to precedence, supported condition keys, canonicalization, reason categories, or audit ordering requires version-boundary review.

## Failure modes and AppError code map
- `KC_LINEAGE_POLICY_CONDITION_INVALID`
- `KC_LINEAGE_POLICY_DENY_ENFORCED`
- `KC_LINEAGE_PERMISSION_DENIED`
- Existing lineage lock/RBAC/query errors remain active:
  - `KC_LINEAGE_LOCK_HELD`
  - `KC_LINEAGE_LOCK_INVALID`
  - `KC_LINEAGE_LOCK_EXPIRED`
  - `KC_LINEAGE_QUERY_FAILED`

## Acceptance tests
- Policy list ordering is deterministic across repeated runs.
- Deny overrides allow for matching conditions.
- No-match paths fail with `KC_LINEAGE_PERMISSION_DENIED`.
- Matching deny paths fail with `KC_LINEAGE_POLICY_DENY_ENFORCED`.
- Audit rows serialize deterministic canonical `details_json`.
- Schema tests validate policy binding and policy audit payload shapes.

## Rollout gate and stop conditions
### Rollout gate
- `cargo test -p kc_core -- lineage`
- `cargo test -p kc_core -- schema_`
- `cargo test -p apps_desktop_tauri -- rpc_schema`
- canonical Rust gate from `knowledgecore-docpack/AGENTS.md`

### Stop conditions
- Any overlay mutation succeeds without lock + RBAC + policy allow.
- Any deterministic ordering rule fails for policy bindings or audit rows.
- Missing schema registry updates or schema validation tests for governance v3.
