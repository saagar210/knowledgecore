# Next Stretch Plan (Post-AD)

## Title
KnowledgeCore Carry-Forward Roadmap: Phases AF, AG, AH, AI, AJ

## Summary
This execution horizon closes all known post-AD carry-forward items:
- AF: trust provider auto-discovery and deterministic tenant policy templates.
- AG: recovery escrow adapter expansion beyond the `aws`/`gcp`/`azure` baseline.
- AH: merge policy expansion beyond `conservative_plus_v3`.
- AI: lineage condition DSL expansion beyond `action` + `doc_id_prefix`.
- AJ: final consolidation and readiness closure.

## Global Invariants
- Local-first remains default; remote adapters are optional.
- UI has no business logic and branches on `AppError.code` only.
- Tauri layer remains thin RPC orchestration only.
- Tier 1 determinism and ordering contracts must not regress.
- Schema changes must update `SCHEMA_REGISTRY.md` and schema validation tests.
- Milestones are merge-as-we-go with stop/fix/rerun gates.

## Phase AF — Trust Discovery and Tenant Templates

### Goal
Add deterministic provider discovery and tenant bootstrap templates on top of existing trust governance/session contracts.

### Scope
- Deterministic provider auto-discovery flow.
- Tenant bootstrap policy templates with deterministic canonicalization.
- Core/CLI/RPC/UI surfaces for discovery/template operations.

### Non-goals
- Non-deterministic provider fallback.
- UI-side trust policy evaluation.

### Acceptance
- Discovery output ordering is deterministic.
- Template application yields canonical claims JSON.
- Existing revocation precedence remains intact.

## Phase AG — Escrow Provider Expansion v4

### Goal
Extend recovery escrow to support additional provider classes (for example HSM/private KMS variants) while preserving deterministic export/verifier behavior.

### Scope
- Catalog-driven provider resolution and ordering.
- Additional provider adapters and availability checks.
- Export/verifier schema closure for expanded provider lists.

### Non-goals
- Secret material persistence in repo-tracked files.
- Non-deterministic provider selection.

### Acceptance
- Provider ordering remains deterministic and test-covered.
- Verifier enforces expanded descriptor invariants deterministically.
- Unsupported providers map to stable AppError codes.

## Phase AH — Merge Policy v4

### Goal
Introduce `conservative_plus_v4` as an opt-in deterministic merge policy beyond v3.

### Scope
- New merge policy semantics and deterministic reason categories.
- Replay-stable decision traces.
- CLI/RPC/UI policy-selection surfacing without UI merge logic.

### Non-goals
- Automatic destructive merges.
- Policy evaluation in UI/Tauri.

### Acceptance
- Unsafe scenarios hard-fail with deterministic reason categories.
- Replay of identical inputs yields identical v4 decisions and traces.

## Phase AI — Lineage Condition DSL v4

### Goal
Extend lineage condition expressiveness while preserving deny-default and deterministic precedence/audit invariants.

### Scope
- Deterministic condition-key expansion beyond `action` + `doc_id_prefix`.
- Core policy evaluation and audit serialization updates.
- CLI/RPC/UI policy-management surface updates.

### Non-goals
- Client-side policy arbitration.
- Distributed/global policy consensus.

### Acceptance
- Overlay mutation still requires lock + RBAC + policy allow.
- Deny override remains deterministic.
- Audit ordering/serialization remains stable.

## Phase AJ — Final Consolidation

### Goal
Close AF–AI with full gate rerun, bench governance checks, and git-hygiene confirmation.

### Scope
- Full canonical gates and schema/RPC reruns.
- Bench gate (`bench run --corpus v1`) twice with baseline policy handling.
- Final readiness and risk-closure note updates.

### Acceptance
- All canonical gates pass.
- Bench policy expectations are documented and satisfied.
- `master` is clean and no milestone branches remain.

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
