# Next Stretch Plan (Post-V)

## Title
KnowledgeCore Remaining Roadmap: Phases W, X, Y, Z

## Summary
This execution horizon closes the remaining deferred scope after Phase V:
- W: managed identity trust (OIDC + device certificates)
- X: recovery escrow provider model (local adapter + AWS first)
- Y: conservative auto-merge expansion with deterministic policy v2
- Z: lineage governance v2 (vault RBAC + scoped lock workflows)

## Global Invariants
- Local-first remains default; remote adapters are optional.
- UI has no business logic and branches on `AppError.code` only.
- Tauri layer remains thin RPC orchestration only.
- Tier 1 determinism and ordering contracts must not regress.
- Schema changes must update `SCHEMA_REGISTRY.md` and schema validation tests.
- Milestones are merge-as-we-go with stop/fix/rerun gates.

## Phase W — Managed Identity Trust v2

### Goal
Introduce OIDC-backed identity sessions and certificate-chain validated device authorship for sync heads.

### Scope
- Trust identity provider/session model.
- Device certificate enrollment and chain verification.
- Sync head v3 authorship/signature chain fields.
- CLI/RPC/UI trust onboarding and status flows.

### Non-goals
- Managed recovery escrow.
- Aggressive sync merge policy changes.

### Acceptance
- Invalid identity/session/cert/signature paths fail with stable AppError codes.
- Sync head v3 serialization and validation are deterministic.

## Phase X — Recovery Escrow v2

### Goal
Add recovery escrow abstraction with local adapter and AWS KMS + Secrets Manager first production adapter.

### Scope
- Escrow provider abstraction and adapter plumbing.
- Recovery manifest v2 escrow metadata.
- CLI/RPC/UI escrow status/enable/rotate/restore flows.
- Export/verifier contract coverage for escrow metadata.

### Non-goals
- Multi-provider production rollout in one phase.
- External secret persistence in repo-tracked state.

### Acceptance
- Local recovery remains functional.
- Escrow-enabled workflows are deterministic and schema-validated.

## Phase Y — Sync Auto-Merge Policy Expansion v2

### Goal
Extend merge policy to `conservative_plus_v2` with explicit safety allowlist and deterministic decision traces.

### Scope
- Core merge policy engine update.
- CLI/RPC/UI policy selection and preview-first workflows.
- Matrix tests for overlap/trust/lock conflicts.

### Non-goals
- Automatic merge without preview.
- Destructive overwrite behavior.

### Acceptance
- Unsafe merges hard-fail with deterministic reasons.
- Allowed merges are deterministic and replay-stable.

## Phase Z — Lineage Governance v2

### Goal
Add vault-DB RBAC governance for lineage overlays and scoped locking for team workflows.

### Scope
- RBAC schema + deterministic permission evaluation.
- Core enforcement on overlay mutation paths.
- CLI/RPC/UI governance surfaces.
- Final horizon consolidation and closure docs.

### Non-goals
- Real-time collaborative conflict resolution models (CRDT/OT).
- Non-deterministic role arbitration.

### Acceptance
- Overlay writes require valid lock + RBAC permission.
- Governance decisions are deterministic and test-covered.

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
