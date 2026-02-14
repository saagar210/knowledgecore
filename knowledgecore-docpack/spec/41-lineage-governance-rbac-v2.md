# Lineage Governance RBAC v2

## Purpose
Define deterministic vault-level RBAC and scoped lock governance for lineage overlay mutation workflows.

## Invariants
- Overlay mutation remains core-only logic and requires both:
  - a valid active lock token
  - an RBAC allow decision for `lineage.overlay.write`
- Permission evaluation is deterministic:
  - sorted by `role_rank ASC`, then `subject_id ASC`, then `role_name ASC`
  - first matching role-permission row decides allow/deny.
- Scoped lock contracts are deterministic and versioned:
  - lock scope kind is one of `doc` or `set`
  - lease timeout uses existing fixed 15-minute lease window.
- UI and Tauri do not evaluate roles, locks, or precedence rules.

## Non-goals
- External identity federation policy decisions.
- Distributed/global lock consensus.
- Automatic role provisioning.

## Interface contracts
- Core role APIs:
  - `lineage_role_grant(conn, subject_id, role_name, granted_by, now_ms)`
  - `lineage_role_revoke(conn, subject_id, role_name)`
  - `lineage_role_list(conn)`
- Core permission APIs:
  - `lineage_permission_decision(conn, subject_id, action)`
  - `ensure_lineage_permission(conn, subject_id, action, doc_id)`
- Core scoped lock APIs:
  - `lineage_lock_acquire_scope(conn, scope_kind, scope_value, owner, now_ms)`
  - `lineage_lock_release_scope(conn, scope_kind, scope_value, token)`
  - `lineage_lock_scope_status(conn, scope_kind, scope_value, now_ms)`
- Surface contracts:
  - CLI governance: `kc_cli lineage role grant|revoke|list`
  - CLI scoped lock: `kc_cli lineage lock acquire-scope`
  - RPC governance: `lineage_role_grant|revoke|list`
  - RPC scoped lock: `lineage_lock_acquire_scope`
- DB schema additions:
  - `lineage_roles`
  - `lineage_permissions`
  - `lineage_role_bindings`
  - `lineage_lock_scopes`

## Determinism and version-boundary rules
- Role binding lists are sorted by:
  - `role_rank ASC`, `subject_id ASC`, `role_name ASC`.
- Permission decision traces always select the first sorted candidate.
- Scoped lock token derivation is deterministic from:
  - `scope_kind`, `scope_value`, `owner`, `now_ms`.
- Any change to precedence order, lock token derivation, or scope-kind enum requires version-boundary review.

## Failure modes and AppError mapping
- `KC_LINEAGE_PERMISSION_DENIED`: subject lacks permission for requested lineage action.
- `KC_LINEAGE_ROLE_INVALID`: invalid role name or role binding operation.
- `KC_LINEAGE_SCOPE_INVALID`: unsupported/malformed scope kind/value.
- Existing lock and query codes remain in force:
  - `KC_LINEAGE_LOCK_HELD`
  - `KC_LINEAGE_LOCK_INVALID`
  - `KC_LINEAGE_LOCK_EXPIRED`
  - `KC_LINEAGE_QUERY_FAILED`

## Acceptance tests
- Role list ordering is deterministic for repeated runs.
- Permission precedence respects role-rank sorting.
- Overlay mutation fails with `KC_LINEAGE_PERMISSION_DENIED` when lock is valid but role missing.
- Scoped lock acquire/status/release path is deterministic.
- Schema tests validate role-binding and scoped-lock status contracts.

## Rollout gate and stop conditions
### Rollout gate
- `cargo test -p kc_core -- lineage`
- `cargo test -p kc_core -- schema_`
- canonical Rust gate from `knowledgecore-docpack/AGENTS.md`.

### Stop conditions
- Any overlay mutation succeeds without RBAC allow + valid lock.
- Any non-deterministic permission ordering for fixed data.
- Missing schema registry updates or schema validation tests.
