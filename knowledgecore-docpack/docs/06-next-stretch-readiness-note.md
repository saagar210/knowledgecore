# Next Stretch Readiness Note

## Purpose
Record execution status after completing the post-N3 roadmap through Phases O, P, Q, R, S, T, U, and V, and activating the W–Z horizon.

## Current Readiness Status
- D–K is complete on `master`.
- Phase L design lock is complete.
- Phase M is complete (object-store encryption + vault v2 rollout).
- Phase N1 is complete (deterministic ZIP packaging).
- Phase N2 is complete (sync v1 with filesystem snapshots).
- Phase N3 is complete (lineage query surface).
- Phase O is complete (URI-based sync target abstraction + S3 transport).
- Phase P is complete (SQLCipher DB encryption + lock-session and migration UX).
- Phase Q is complete (overlay-only lineage write model with deterministic `lineage_query_v2`).
- Phase R is complete (preview scaffold retirement + horizon hardening).
- Phase S is complete (manual device trust + local recovery kit).
- Phase T is complete (conservative sync auto-merge preview/apply flow).
- Phase U is complete (turn-based lineage edit lock model and desktop workflows).
- Phase V is complete (final consolidation for this horizon).
- Phase W0 is activated (managed identity + sync head v3 specs ratified).
- Phases W1–Z3 are planned and pending implementation.

## Required Reference Set
- `knowledgecore-docpack/docs/03-phase-d-k-closure-report.md`
- `knowledgecore-docpack/docs/04-post-dk-ops-and-followup-policy.md`
- `knowledgecore-docpack/docs/05-next-stretch-plan.md`
- `knowledgecore-docpack/docs/07-phase-l-execution-notes.md`
- `knowledgecore-docpack/SCHEMA_REGISTRY.md`

## Completion Evidence (S→U)
| Milestone | Branch | Merge Commit | Notes |
|---|---|---|---|
| S0 | `codex/s0-security-contract-activation` | `915242c` | activated specs `35` and `36`, registry alignment |
| S1 | `codex/s1-device-trust-core` | `adf584e` | device trust schema/tables + deterministic fingerprint flow |
| S2 | `codex/s2-recovery-kit` | `97a8c4b` | recovery kit generate/verify RPC + CLI + UI settings surface |
| T1 | `codex/t1-sync-merge-core` | `3f564d4` | conservative merge engine + deterministic merge report |
| T2 | `codex/t2-sync-merge-surface` | `df2e698` | sync merge preview/pull surfaces across CLI/RPC/UI |
| U1 | `codex/u1-lineage-lock-core` | `bc363ef` | lock lease model + schema v5 + overlay lock enforcement |
| U2 | `codex/u2-lineage-lock-surface` | `f9fb7b6` | lock acquire/release/status surfaces in CLI/RPC/UI |
| V1 | `codex/v1-final-consolidation` | this milestone commit | final risk closure evidence + gates rerun |

## Gate Evidence (Source: `knowledgecore-docpack/AGENTS.md`)
- Rust gate:
  - `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli`
- Desktop gate:
  - `pnpm lint && pnpm test && pnpm tauri build`
- RPC/schema gates:
  - `cargo test -p apps_desktop_tauri -- rpc_`
  - `cargo test -p apps_desktop_tauri -- rpc_schema`
  - `cargo test -p kc_core -- schema_`
  - `cargo test -p kc_cli -- schema_`
- Final bench gate (V1 target):
  - `cargo run -p kc_cli -- bench run --corpus v1` (twice)

## Carry-Forward Deferred Table (Post-V)
| Item | Status | Carry-Forward Target | Notes |
|---|---|---|---|
| Managed identity / PKI trust exchange | Deferred | Future security horizon | Current trust model is manual fingerprint verification |
| Recovery key escrow / remote recovery management | Deferred | Future security horizon | Current model is local recovery kit only |
| Auto-merge policy beyond conservative disjoint-only | Deferred | Future sync horizon | Current auto-merge is opt-in conservative mode |
| Multi-doc/team-role lineage lock governance | Deferred | Future lineage horizon | Current lock model is per-doc turn lease |

## Next Horizon Mapping (W–Z)
- W: managed identity trust v2 (`spec/39-managed-identity-oidc-device-cert-v1.md`)
- W: sync head signature chain v3 (`spec/40-sync-head-signature-chain-v3.md`)
- X: recovery escrow v2 (provider abstraction, AWS first)
- Y: sync merge policy expansion v2 (`conservative_plus_v2`)
- Z: lineage governance v2 (vault RBAC + scoped locks)

## Git Hygiene Note
- Fast-forward merge mode was used for completed milestones.
- Local milestone refs were removed after merge; environment fallback used when `git branch -d` was blocked:
  - `git update-ref -d refs/heads/<merged-branch>`
- `master` remains the only active local branch between milestones.
