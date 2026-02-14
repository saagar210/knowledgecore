# RUNBOOK_CODEX.md

## Purpose
Operational runbook for GPT-5.3-Codex (High Reasoning) to implement KnowledgeCore Desktop milestone-by-milestone, enforcing gates, determinism, and schema registry updates.

## Invariants
- Follow `AGENTS.md` and `plan/*` strictly.
- Implement **one milestone at a time**. Do not mix milestones unless the plan explicitly says so.
- Hard stop on any failing verification command; fix first.
- Update `SCHEMA_REGISTRY.md` whenever you add/change any schema (including RPC request/response types).

## Acceptance Tests
- Each milestone ends with green gates and a PR-ready summary.
- Schema changes include updated registry entries and validation tests.

## Codex execution protocol (per milestone)
1) Create branch: `milestone/<NN>-<short-name>`
2) Implement only the milestone scope from `plan/01-step-by-step-implementation-plan.md`
3) Run gates from `plan/00-milestones-and-gates.md`
4) If any gate fails: stop, diagnose, fix, rerun until green
5) Summarize changes (by crate) + list any schema changes
6) Open PR using the template below
7) Proceed only when PR is reviewable and all gates are green

## Reasoning level guidance
- Use **High Reasoning** for: deterministic algorithms, tie-break rules, schema design/versioning, export/verifier correctness, cross-crate boundaries.
- Use **Medium Reasoning** for: mechanical file creation, wiring, repetitive unit tests.

## Commit message template
- `milestone(<NN>): <short summary>`
- Body: why, what changed, tests run (exact commands)

## PR template
- Title: `Milestone <NN>: <short summary>`
- Description:
  - Scope implemented:
  - Out of scope (explicit):
  - Schemas updated (if any):
  - Determinism impacts (Tier 1 / Tier 2 / Tier 3):
  - Tests run (commands + results):

## Mandatory registry update rule
If you add/modify: vault.json, AppError, Locator v1, Export manifest, Verifier report, Trace log, or RPC types:
- update `SCHEMA_REGISTRY.md`
- add/update schema validation tests

## Failure handling
When a gate fails: capture output, isolate root cause, fix minimal correct change, rerun gate until green.

## Definition of done (milestone)
- Gates green
- No boundary violations
- Specs/registry updated where required
- Acceptance tests exist and pass
