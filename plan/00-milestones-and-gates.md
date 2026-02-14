# plan/00-milestones-and-gates.md

## Purpose
Provide the ordered execution plan and gates for phases 0, A–K. Each milestone includes goal, crates touched, invariants, verification commands, acceptance signals, and Stop/Go gate.

## Invariants
- Must maintain boundaries from AGENTS.md.
- Must satisfy determinism tiers.
- Must ensure CLI verification exists before desktop UI begins.

## Acceptance Tests
- Each milestone specifies executable commands and measurable acceptance signals.

## Milestone Map

### Phase 0: Repo bootstrap and boundary scaffolding
- Goal: workspace layout, minimal crate skeletons, CI commands wired.
- Crates: all
- Gate:
  - `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli` (initial trivial tests)
  - `pnpm lint && pnpm test && pnpm tauri build` (UI scaffold may be minimal and tests trivial)

### Phase A: Spec pack v1 and schema registry stabilization
- Goal: specs exist and are internally consistent; schema registry complete.
- Gate:
  - spec lint (markdown check optional)
  - `cargo test -p kc_core -- canonical_json_*` once implemented

### Phase B: Vault substrate (SQLite + object store + vault.json)
- Goal: open/init vault, apply migrations, object store read/write/verify.
- Crates: kc_core, kc_cli
- Gate:
  - `cargo test -p kc_core -p kc_cli`

### Phase C: Ingest jobs + timestamp resolution + event log
- Goal: scan folder ingest and inbox watcher semantics; effective_ts stored.
- Crates: kc_core, kc_cli
- Gate:
  - `cargo test -p kc_core -p kc_cli`

### Phase D: Canonicalization pipelines (MD/HTML/PDFium/OCR) + provenance
- Goal: canonical text produced and stored with markers; toolchain pinned.
- Crates: kc_extract, kc_core, kc_cli
- Gate:
  - `cargo test -p kc_extract -p kc_core -p kc_cli`
  - golden extraction tests (once fixtures exist)

### Phase E: Chunking + FTS5 index
- Goal: deterministic chunking + FTS rebuild/query candidate retrieval.
- Crates: kc_core, kc_index, kc_cli
- Gate:
  - `cargo test -p kc_core -p kc_index -p kc_cli`

### Phase F: LanceDB + embeddings + hybrid retrieval merge
- Goal: vector index build/query + deterministic merge ordering.
- Crates: kc_index, kc_core, kc_cli
- Gate:
  - `cargo test -p kc_index -p kc_core -p kc_cli`

### Phase G: Locator resolver + snippet rendering
- Goal: strict resolver and display-only snippet.
- Crates: kc_core, kc_cli
- Gate:
  - `cargo test -p kc_core -p kc_cli`

### Phase H: Export bundles + verifier
- Goal: deterministic manifest + verifier with stable exit codes and report ordering.
- Crates: kc_core, kc_cli
- Gate:
  - `cargo test -p kc_core -p kc_cli`
  - export/verify golden tests

### Phase I: Ask mode + trace logs
- Goal: retrieved-only ask + citation enforcement + trace log schema.
- Crates: kc_ask, kc_core, kc_cli
- Gate:
  - `cargo test -p kc_ask -p kc_core -p kc_cli`

### Phase J: Desktop UI (full feature)
- Goal: UI implements full scope using RPC only; no business logic.
- Crates: apps/desktop
- Gate:
  - `pnpm lint && pnpm test && pnpm tauri build`
  - integration smoke tests against local vault

### Phase K: Packaging + ops + performance regressions
- Goal: dependency strategy validated; maintenance tools; benchmark harness.
- Gate:
  - all Rust tests + UI build
  - bench smoke command exists

## Explicit Deferrals (to prevent scope creep)
- Encryption at rest
- Deterministic ZIP packaging
- Cross-device sync
- Advanced lineage UI
