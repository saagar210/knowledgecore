# Next Stretch Plan (Post-Z)

## Title
KnowledgeCore Remaining Roadmap: Phases AA, AB, AC, AD

## Summary
This execution horizon closes the remaining deferred scope after Z3:
- AA: policy-driven OIDC provider governance automation.
- AB: recovery escrow provider expansion beyond AWS-first.
- AC: merge policy expansion beyond `conservative_plus_v2`.
- AD: lineage governance conditions beyond role-rank precedence.

## Global Invariants
- Local-first remains default; remote adapters are optional.
- UI has no business logic and branches on `AppError.code` only.
- Tauri layer remains thin RPC orchestration only.
- Tier 1 determinism and ordering contracts must not regress.
- Schema changes must update `SCHEMA_REGISTRY.md` and schema validation tests.
- Milestones are merge-as-we-go with stop/fix/rerun gates.

## Phase AA — Trust Governance Automation

### Goal
Introduce deterministic OIDC provider lifecycle management with explicit policy controls and session revocation support.

### Scope
- Provider CRUD and enable/disable lifecycle.
- Claim requirement policy and max clock-skew controls.
- Session revocation state and precedence.
- CLI/RPC/UI operator surfaces for provider governance.

### Non-goals
- New trust signature formats.
- Replacing existing trust enrollment primitives.

### Acceptance
- Disabled providers cannot complete sessions.
- Revoked sessions are never selected as active author identities.
- Provider policy evaluation is deterministic and test-covered.

## Phase AB — Recovery Escrow Expansion

### Goal
Add deterministic multi-provider recovery escrow management with ordered provider descriptors and rotation orchestration.

### Scope
- Multi-provider escrow configuration model.
- Provider adapters for AWS, GCP, Azure (runtime-available adapters may remain env-gated).
- CLI/RPC/UI provider add/list/rotate-all surfaces.
- Export/verifier alignment for escrow descriptor lists.

### Non-goals
- External secret persistence in repo-tracked files.
- Non-deterministic provider fallback ordering.

### Acceptance
- Provider ordering is stable (`aws`, `gcp`, `azure`).
- Manifest escrow metadata is deterministic and verifier-enforced.
- Failures map to stable AppError codes.

## Phase AC — Merge Policy v3

### Goal
Extend sync merge preview/apply policy with explicit RBAC conflict precondition and deterministic decision traces.

### Scope
- New policy `conservative_plus_v3`.
- Policy selection from CLI/RPC/UI without UI merge logic.
- Safety matrix tests for overlap, trust, lock, and RBAC conflicts.

### Non-goals
- Automatic destructive merges.
- Policy evaluation in UI/Tauri.

### Acceptance
- Unsafe merges hard-fail with deterministic reason categories.
- Replay of identical inputs yields identical merge decisions and traces.

## Phase AD — Lineage Condition Governance

### Goal
Layer deterministic condition-based policy evaluation over RBAC for lineage overlay mutations, with audit stability.

### Scope
- Policy condition model (allow/deny) and subject bindings.
- Deterministic precedence (`priority`, `policy_id`, `subject_id`).
- CLI/RPC/UI policy management surfaces.
- Audit event generation and schema hardening.

### Non-goals
- Distributed lock consensus.
- Client-side governance arbitration.

### Acceptance
- Overlay mutation requires lock + RBAC + condition allow.
- Explicit deny overrides allow deterministically.
- Audit ordering and serialization remain stable.

## Canonical Gates
- Rust: `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli`
- Desktop: `pnpm lint && pnpm test && pnpm tauri build`
- RPC/schema:
  - `cargo test -p apps_desktop_tauri -- rpc_`
  - `cargo test -p apps_desktop_tauri -- rpc_schema`
  - `cargo test -p kc_core -- schema_`
  - `cargo test -p kc_cli -- schema_`
- Final consolidation bench:
  - `cargo run -p kc_cli -- bench run --corpus v1` (twice)
