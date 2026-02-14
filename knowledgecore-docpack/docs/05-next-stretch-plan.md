# Next Stretch Plan (Post-R)

## Title
KnowledgeCore Remaining Roadmap: Phases S, T, U, V

## Summary
This execution horizon closes the remaining deferred scope after O–R:
- S: security foundations (manual device trust + local recovery kit)
- T: conservative sync auto-merge (preview-first)
- U: collaborative lineage with turn-based locks
- V: final hardening and closure

## Global Invariants
- Local-first remains default; remote adapters are optional.
- UI has no business logic and branches on `AppError.code` only.
- Tauri layer remains thin RPC orchestration only.
- Tier 1 determinism and ordering contracts must not regress.
- Schema changes must update `SCHEMA_REGISTRY.md` and schema validation tests.
- Milestones are merge-as-we-go with stop/fix/rerun gates.

## Phase S — Security Foundations

### Goal
Introduce manual device-key verification trust and local recovery-kit workflows.

### Scope
- Device trust APIs and schema.
- Trust author metadata in sync heads.
- Recovery bundle generate/verify/status APIs.
- CLI/RPC/UI settings surface for trust and recovery.

### Non-goals
- Managed identity.
- Shamir secret sharing.
- Remote escrow.

### Acceptance
- Unverified devices cannot author accepted remote heads.
- Fingerprint verification is explicit and deterministic.
- Recovery bundle verification hard-fails on mismatch/tamper.

## Phase T — Sync Conservative Auto-Merge

### Goal
Enable opt-in conservative merge where only disjoint deterministic changes are auto-applied.

### Scope
- `sync_merge` preview report contract and deterministic ordering.
- CLI surface for `sync merge-preview` and `sync pull --auto-merge conservative`.
- RPC and UI settings preview wiring.

### Non-goals
- Aggressive conflict resolution.
- Hidden/implicit merge behavior.

### Acceptance
- Overlap hard-fails with `KC_SYNC_MERGE_NOT_SAFE`.
- Disjoint sets merge predictably with no silent overwrite.

## Phase U — Collaborative Lineage (Turn-Based Lock)

### Goal
Add edit-lock semantics for lineage overlays with explicit acquire/release/status.

### Scope
- Per-doc lock table and lock APIs.
- Overlay mutation paths gated by valid lock token.
- CLI/RPC/UI lock workflows.

### Non-goals
- Real-time collaborative merge.
- CRDT/OT editing models.

### Acceptance
- Lock contention and expiration codes are deterministic.
- Overlay mutations without valid lock token fail.

## Phase V — Final Consolidation

### Goal
Close risks/follow-ups, run final full gates, and leave planning-ready `master`.

### Scope
- Readiness and operations docs updates.
- Final canonical Rust/desktop/schema/RPC gates.
- Bench smoke run twice.

### Acceptance
- No unresolved S–U follow-ups in this horizon.
- `master` clean with branch hygiene normalized.

## Canonical Gates
- Rust: `cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli`
- Desktop: `pnpm lint && pnpm test && pnpm tauri build`
- RPC/schema: `cargo test -p apps_desktop_tauri -- rpc_` and `cargo test -p apps_desktop_tauri -- rpc_schema`
- Bench smoke: `cargo run -p kc_cli -- bench run --corpus v1` (twice at final consolidation)
