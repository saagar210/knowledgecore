# Phase L–Z Execution Notes

## Purpose
Track baseline, milestone progression, gate evidence, and risk/follow-up closure across Phases L through Z.

## Baseline
- Baseline branch: `master`
- Baseline SHA for Phase L kickoff: `51a8b2b`

## Verification Command Sources
- `/Users/d/Projects/knowledgecore/knowledgecore-docpack/AGENTS.md`
- `/Users/d/Projects/knowledgecore/knowledgecore-docpack/SCHEMA_REGISTRY.md`
- `/Users/d/Projects/knowledgecore/knowledgecore-docpack/CHECKLIST_VERIFICATION.md`

## Milestone Ledger (Merge-As-We-Go)
| Milestone | Branch | Commit | Merge Mode | Status |
|---|---|---|---|---|
| L | `codex/l-phase-l-design-lock` | `51a8b2b` | ff-only | Complete |
| M0 | `codex/m0-post-l-hygiene` | merged prior | ff-only | Complete |
| M1 | `codex/m1-phase-m-core-encryption` | `6db6b39` | ff-only | Complete |
| M2 | `codex/m2-phase-m-ux-migration` | `07009f6` | ff-only | Complete |
| N1 | `codex/n1-phase-n-zip-packaging` | `56bd485` | ff-only | Complete |
| N2 | `codex/n2-phase-n-sync-filesystem` | `6d1162d` | ff-only | Complete |
| N3 | `codex/n3-phase-n-lineage-ui` | `ef29732` | ff-only | Complete |
| O0 | `codex/o0-contract-realignment` | `7f0bf3c` | ff-only | Complete |
| O1 | `codex/o1-sync-transport-foundation` | `a5e3b67` | ff-only | Complete |
| O2 | `codex/o2-sync-s3-execution` | `c67dcb3` | ff-only | Complete |
| O3 | `codex/o3-sync-s3-surface` | `399bbd6` | ff-only | Complete |
| P1 | `codex/p1-sqlcipher-core` | `7bac439` | ff-only | Complete |
| P2 | `codex/p2-sqlcipher-migration-ux` | `0b9b72c` | ff-only | Complete |
| P3 | `codex/p3-sqlcipher-export-verify` | `e5f4b35` | ff-only | Complete |
| Q1 | `codex/q1-lineage-overlays-core` | `6e27675` | ff-only | Complete |
| Q2 | `codex/q2-lineage-overlays-ui` | `7477f47` | ff-only | Complete |
| R1 | `codex/r1-preview-retirement` | `ee4098d` | ff-only | Complete |
| R2 | `codex/r2-final-consolidation` | `34a66b5` | ff-only | Complete |
| S0 | `codex/s0-security-contract-activation` | `915242c` | ff-only | Complete |
| S1 | `codex/s1-device-trust-core` | `adf584e` | ff-only | Complete |
| S2 | `codex/s2-recovery-kit` | `97a8c4b` | ff-only | Complete |
| T1 | `codex/t1-sync-merge-core` | `3f564d4` | ff-only | Complete |
| T2 | `codex/t2-sync-merge-surface` | `df2e698` | ff-only | Complete |
| U1 | `codex/u1-lineage-lock-core` | `bc363ef` | ff-only | Complete |
| U2 | `codex/u2-lineage-lock-surface` | `f9fb7b6` | ff-only | Complete |
| V1 | `codex/v1-final-consolidation` | `b4ef40b` | ff-only | Complete |
| W0 | `codex/w0-trust-contract-activation` | `c99421e` | ff-only | Complete |
| W1 | `codex/w1-trust-core-v2` | `a2a8a44` | ff-only | Complete |
| W2 | `codex/w2-trust-surface` | `6fd0f2e` | ff-only | Complete |
| W3 | `codex/w3-trust-schema-hardening` | `9ae7e4d` | ff-only | Complete |
| X1 | `codex/x1-recovery-escrow-core` | `b968347` | ff-only | Complete |
| X2 | `codex/x2-recovery-escrow-surface` | `4e6391e` | ff-only | Complete |
| X3 | `codex/x3-recovery-escrow-verifier` | `5dc83f4` | ff-only | Complete |
| Y1 | `codex/y1-sync-merge-policy-v2-core` | `978d18e` | ff-only | Complete |
| Y2 | `codex/y2-sync-merge-policy-v2-surface` | `c812c30` | ff-only | Complete |
| Y3 | `codex/y3-sync-merge-policy-v2-tests` | `dc099ca` | ff-only | Complete |
| Z1 | `codex/z1-lineage-rbac-core` | `4bac68c` | ff-only | Complete |
| Z2 | `codex/z2-lineage-rbac-surface` | `b2e15e1` | ff-only | Complete |
| Z3 | `codex/z3-final-consolidation` | `(this commit)` | ff-only | Complete |

## Major Contract Promotions
- Encryption-at-rest active contract: `knowledgecore-docpack/spec/27-encryption-at-rest-v1.md`
- Deterministic ZIP active contract: `knowledgecore-docpack/spec/28-deterministic-zip-packaging-v1.md`
- Sync v1 active contract: `knowledgecore-docpack/spec/29-sync-v1-filesystem-snapshots.md`
- Lineage v1 read-only baseline: `knowledgecore-docpack/spec/30-advanced-lineage-ui-v1.md`
- Sync S3 transport contract: `knowledgecore-docpack/spec/31-sync-s3-transport-v1.md`
- SQLCipher contract: `knowledgecore-docpack/spec/32-sqlite-encryption-sqlcipher-v1.md`
- Passphrase trust contract: `knowledgecore-docpack/spec/33-cross-device-passphrase-trust-v1.md`
- Lineage overlays contract: `knowledgecore-docpack/spec/34-lineage-overlays-v1.md`
- Device trust manual verification contract: `knowledgecore-docpack/spec/35-device-trust-manual-verify-v1.md`
- Local recovery kit contract: `knowledgecore-docpack/spec/36-local-recovery-kit-v1.md`
- Conservative auto-merge contract: `knowledgecore-docpack/spec/37-sync-conservative-auto-merge-v1.md`
- Turn-based lineage lock contract: `knowledgecore-docpack/spec/38-lineage-collab-turn-lock-v1.md`
- Managed identity trust v2 contract: `knowledgecore-docpack/spec/39-managed-identity-oidc-device-cert-v1.md`
- Sync head signature chain v3 contract: `knowledgecore-docpack/spec/40-sync-head-signature-chain-v3.md`
- Lineage governance RBAC v2 contract: `knowledgecore-docpack/spec/41-lineage-governance-rbac-v2.md`

## Verification Summary
- Canonical Rust gate passed on completed milestones:
  - `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli`
- Desktop gate passed on desktop-affecting milestones:
  - `pnpm lint && pnpm test && pnpm tauri build`
- Schema and RPC checks passed where applicable:
  - `cargo test -p kc_core -- schema_`
  - `cargo test -p kc_cli -- schema_`
  - `cargo test -p apps_desktop_tauri -- rpc_`
  - `cargo test -p apps_desktop_tauri -- rpc_schema`

## Risk Closure Mapping
| Risk | Closure Evidence |
|---|---|
| SQLCipher portability/build drift | SQLCipher path compiled in canonical Rust + desktop builds; migration tests green |
| S3 lock/consistency race | deterministic lock protocol + conflict artifact tests; no silent overwrite behavior |
| Passphrase trust mismatch | head trust metadata validation with deterministic `KC_SYNC_KEY_MISMATCH` paths |
| Recovery kit misuse/tamper | deterministic manifest/checksum verification with explicit mismatch error tests |
| Sync auto-merge false positives | conservative disjoint-only merge policy + preview report tests |
| Lineage lock contention drift | fixed 15-minute lease semantics with lock token validation and expiration tests |
| OIDC/provider variability | deterministic claim subset normalization + certificate-chain hash tests |
| Escrow provider dependency drift | provider abstraction with deterministic unavailable/auth failure paths |
| Merge policy regression risk | `conservative_plus_v2` safety matrix and replay-stability tests |
| RBAC privilege drift | deterministic role-rank precedence tests + deny-default permission checks |
| UI/Tauri business-logic leakage | core-only merge/lineage lock logic + RPC/UI thin-surface tests |
| Schema drift across Rust/UI | schema registry updates plus schema and RPC request/response tests per milestone |

## Git Hygiene Notes
- Per-milestone fast-forward merges were used.
- Local `codex/*` refs were deleted after merge.
- Deletion fallback used in this environment:
  - `git update-ref -d refs/heads/<branch>`
- `master` was kept as the only active local branch between milestones.
