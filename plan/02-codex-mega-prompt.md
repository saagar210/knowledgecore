# plan/02-codex-mega-prompt.md

## Purpose
Single copy/paste mega-prompt for GPT-5.3-Codex (High Reasoning) to implement KnowledgeCore Desktop milestone-by-milestone, obey boundaries, enforce determinism, and stop on failures.

## Invariants
- Must follow `AGENTS.md`, all `docs/*`, and all `spec/*` as source of truth.
- Must not implement business logic in UI or Tauri.
- Must update `SCHEMA_REGISTRY.md` whenever schemas change.
- Must stop on any failing gate and fix before continuing.

## Acceptance Tests
- Codex follows milestones and produces PR-ready increments with green gates.

## Codex Mega-Prompt (copy/paste below)

You are implementing KnowledgeCore Desktop as a greenfield repo. Source of truth is:
- AGENTS.md
- RUNBOOK_CODEX.md
- CHECKLIST_VERIFICATION.md
- SCHEMA_REGISTRY.md
- docs/*
- spec/*
- plan/00-milestones-and-gates.md
- plan/01-step-by-step-implementation-plan.md

Rules:
1) No business logic in UI. UI branches on AppError.code only.
2) Tauri backend is thin RPC only. No ranking/chunking/locator/export logic.
3) Tier 1 determinism must be enforced exactly as specified.
4) Tier 2 outputs (PDF/OCR) are version-bounded with pinned toolchain recorded.
5) Stop on failure: if any verification command fails, stop, diagnose, fix, and rerun until green.
6) Update SCHEMA_REGISTRY.md for any schema change and add schema validation tests.

Execution procedure:
- Implement milestones in order: Phase 0, A, B, C, D, E, F, G, H, I, J, K.
- For each milestone:
  a) Create branch: milestone/<NN>-<name>
  b) Implement only the milestone scope.
  c) Run the milestone gates from plan/00.
  d) If fail: fix and rerun.
  e) Summarize changes by crate and list any schema changes.
  f) Prepare PR description using RUNBOOK_CODEX.md template.

Milestone-by-milestone instructions:
- Follow plan/01-step-by-step-implementation-plan.md tasks in order.
- Do not skip tests. Do not defer determinism tests.

Verification commands:
- Always run:
  - cargo test -p kc_core -p kc_extract -p kc_index -p kc_ask -p kc_cli
- When UI begins:
  - pnpm lint && pnpm test && pnpm tauri build

Schema discipline:
- If you add/modify:
  - vault.json schema
  - locator schema
  - AppError schema
  - export manifest schema
  - verifier report schema
  - trace log schema
  - RPC request/response types
then you must update SCHEMA_REGISTRY.md and add validation tests.

Stop conditions:
- Any gate fails.
- Any Tier 1 deterministic rule cannot be implemented as specified.
- Any boundary violation is introduced (UI logic, Tauri logic).
In those cases: stop, explain the issue, propose the smallest correction, implement it, rerun gates, and only then proceed.

Output expectations:
- Produce concrete code, not pseudocode.
- Keep modules aligned to repo layout.
- Keep algorithms aligned to specs.
- Keep ordering rules explicit and tested.
