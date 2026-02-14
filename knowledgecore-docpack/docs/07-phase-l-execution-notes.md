# Phase L–N Execution Notes

## Purpose
Track baseline, milestone progression, gate evidence, and risk/follow-up closure across Phase L through Phase N3.

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
| L | `codex/l-phase-l-design-lock` | `51a8b2b` baseline + merged output | ff-only | Complete |
| M0 | `codex/m0-post-l-hygiene` | merged prior | ff-only | Complete |
| M1 | `codex/m1-phase-m-core-encryption` | `6db6b39` | ff-only | Complete |
| M2 | `codex/m2-phase-m-ux-migration` | `07009f6` | ff-only | Complete |
| N1 | `codex/n1-phase-n-zip-packaging` | `56bd485` | ff-only | Complete |
| N2 | `codex/n2-phase-n-sync-filesystem` | `6d1162d` | ff-only | Complete |
| N3 | `codex/n3-phase-n-lineage-ui` | `ef29732` | ff-only | Complete |

## Major Contract Promotions
- Encryption metadata promoted from draft to active:
  - `knowledgecore-docpack/spec/27-encryption-at-rest-v1.md`
- ZIP packaging metadata promoted from draft to active:
  - `knowledgecore-docpack/spec/28-deterministic-zip-packaging-v1.md`
- Sync snapshot schema promoted from draft to active:
  - `knowledgecore-docpack/spec/29-sync-v1-filesystem-snapshots.md`
- Lineage query schema promoted from draft to active:
  - `knowledgecore-docpack/spec/30-advanced-lineage-ui-v1.md`

## Verification Summary
- Canonical Rust gate passed for each milestone:
  - `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli`
- Desktop gate passed for desktop-affecting milestones:
  - `pnpm lint && pnpm test && pnpm tauri build`
- Schema and RPC checks passed where applicable:
  - `cargo test -p kc_core -- schema_`
  - `cargo test -p kc_cli -- schema_`
  - `cargo test -p apps_desktop_tauri -- rpc_`
  - `cargo test -p apps_desktop_tauri -- rpc_schema`

## Risk Closure Mapping
| Risk | Closure Evidence |
|---|---|
| Native dependency variability | Toolchain identity captured in export metadata and verifier checks (M/N gates green) |
| Vault schema breakage (v2) | v1 read compatibility and schema tests in `kc_core` |
| Deterministic ZIP drift | Stable ZIP policy (`stored`, fixed mtime, normalized mode) with repeatability tests |
| Sync overwrite/conflict complexity | Hard-fail conflict artifact path; no auto-merge behavior |
| Lineage business logic leak to UI | Core-only lineage assembly + RPC/UI contract tests + checklist update |

## Git Hygiene Notes
- Per-milestone fast-forward merges were used.
- Local branch deletion remains blocked by execution policy for this environment when running:
  - `git branch -d <branch>`
- Manual cleanup instruction remains valid for unrestricted shells.
